use rocket::form::Form;
use rocket::response::Redirect;
use rocket::serde::{Deserialize, Serialize};
use rocket_db_pools::Connection;
use rocket_dyn_templates::{context, Template};

use crate::db::models::{Item, List};
use crate::db::{DataError, WishlistDb};
use crate::web::{self, WebError};

#[derive(FromForm, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct CreateItem<'r> {
    pub title: &'r str,
    pub description: &'r str,
}

#[derive(FromForm, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct EditItem<'r> {
    pub title: &'r str,
    pub description: &'r str,
}

#[get("/lists/<list_key>/items")]
pub async fn index(
    mut db: Connection<WishlistDb>,
    list_key: &str,
) -> Result<Template, WebError<Template>> {
    let list = List::find_by_key(&mut db, list_key)
        .await?
        .ok_or(WebError::NotFound(Template::render("error/404", ())))?;

    let items = Item::all_by_list(&mut db, list.id)
        .await
        .unwrap_or(vec![])
        .into_iter()
        .map(|item| {
            let link = uri!(show(&list.key, item.id)).to_string();
            context! {
                id: item.id,
                title: item.title,
                description: item.description,
                link,
            }
        })
        .collect::<Vec<_>>();

    Ok(Template::render(
        "items/index",
        context! { list, items: items },
    ))
}

#[get("/lists/<list_key>/items/new")]
pub async fn new(
    mut db: Connection<WishlistDb>,
    list_key: &str,
) -> Result<Template, WebError<Template>> {
    let list = List::find_by_key(&mut db, list_key)
        .await?
        .ok_or(WebError::NotFound(Template::render("error/404", ())))?;

    Ok(Template::render("items/new", context! { list }))
}

#[post("/lists/<list_key>/items", format = "form", data = "<item>")]
pub async fn create(
    mut db: Connection<WishlistDb>,
    list_key: &str,
    item: Form<CreateItem<'_>>,
) -> Result<Redirect, WebError<Template>> {
    let list = List::find_by_key(&mut db, list_key)
        .await?
        .ok_or(WebError::NotFound(Template::render("error/404", ())))?;

    match Item::create(&mut db, list.id, item.title, item.description).await {
        Ok(item) => Ok(Redirect::to(uri!(web::items::show(list.key, item.id)))),
        Err(DataError::Validation(e)) => Err(WebError::Invalid(Template::render(
            "items/new",
            context! {
                list,
                item: context! {
                    title: item.title,
                    description: item.description,
                },
                error_message: "Fix your errors",
                errors: e,
            },
        ))),
        Err(e) => Err(WebError::Invalid(Template::render(
            "items/new",
            context! {
                list,
                item: context! {
                    title: item.title,
                    description: item.description,
                },
                error_message: e.to_string()
            },
        ))),
    }
}

#[get("/lists/<list_key>/items/<id>", rank = 2)]
pub async fn show(
    mut db: Connection<WishlistDb>,
    list_key: &str,
    id: i64,
) -> Result<Template, WebError<Template>> {
    let list = List::find_by_key(&mut db, list_key)
        .await?
        .ok_or(WebError::NotFound(Template::render("error/404", ())))?;

    let item = Item::find_by_id(&mut db, id).await?;

    Ok(Template::render("items/show", context! { list, item }))
}

#[get("/lists/<list_key>/items/<id>/edit")]
pub async fn edit(
    mut db: Connection<WishlistDb>,
    list_key: &str,
    id: i64,
) -> Result<Template, WebError<Template>> {
    let list = List::find_by_key(&mut db, list_key)
        .await?
        .ok_or(WebError::NotFound(Template::render("error/404", ())))?;

    let item = Item::find_by_id(&mut db, id).await?;

    Ok(Template::render("items/edit", context! { list, item }))
}

#[put("/lists/<list_key>/items/<id>", format = "form", data = "<item>")]
pub async fn update(
    mut db: Connection<WishlistDb>,
    list_key: &str,
    id: i64,
    item: Form<EditItem<'_>>,
) -> Result<Redirect, WebError<Template>> {
    let list = List::find_by_key(&mut db, list_key)
        .await?
        .ok_or(WebError::NotFound(Template::render("error/404", ())))?;

    let mut old_item = Item::find_by_id(&mut db, id)
        .await?
        .ok_or(WebError::NotFound(Template::render("error/404", ())))?;

    match old_item
        .update(&mut db, &item.title, &item.description)
        .await
    {
        Ok(item) => Ok(Redirect::to(uri!(web::items::show(list.key, item.id)))),
        Err(DataError::Validation(e)) => Err(WebError::Invalid(Template::render(
            "items/edit",
            context! {
                list,
                item: context! {
                    id,
                    title: item.title,
                    description: item.description,
                },
                error_message: "Fix your errors",
                errors: e,
            },
        ))),
        Err(e) => Err(WebError::Invalid(Template::render(
            "items/edit",
            context! {
                list,
                item: context! {
                    id,
                    title: item.title,
                    description: item.description,
                },
                error_message: e.to_string()
            },
        ))),
    }
}

#[delete("/lists/<list_key>/items/<id>")]
pub async fn destroy(
    mut db: Connection<WishlistDb>,
    list_key: &str,
    id: i64,
) -> Result<Redirect, WebError<Template>> {
    let list = List::find_by_key(&mut db, list_key)
        .await?
        .ok_or(WebError::NotFound(Template::render("error/404", ())))?;

    let mut item = Item::find_by_id(&mut db, id)
        .await?
        .ok_or(WebError::NotFound(Template::render("error/404", ())))?;

    item.destroy(&mut db).await?;

    Ok(Redirect::to(uri!(web::items::index(list.key))))
}
