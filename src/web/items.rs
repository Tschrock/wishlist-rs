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

#[get("/lists/<list_id>/items")]
pub async fn index(
    mut db: Connection<WishlistDb>,
    list_id: i64,
) -> Result<Template, WebError<Template>> {
    let list = List::find_by_id(&mut db, list_id)
        .await?
        .ok_or(WebError::NotFound(Template::render("error/404", ())))?;

    let items = Item::all_by_list(&mut db, list_id)
        .await
        .unwrap_or(vec![])
        .into_iter()
        .map(|item| {
            context! {
                id: item.id,
                title: item.title,
                description: item.description,
                link: uri!(show(list.id, item.id)).to_string(),
            }
        })
        .collect::<Vec<_>>();

    Ok(Template::render(
        "items/index",
        context! { list, items: items },
    ))
}

#[get("/lists/<list_id>/items/new")]
pub async fn new(
    mut db: Connection<WishlistDb>,
    list_id: i64,
) -> Result<Template, WebError<Template>> {
    let list = List::find_by_id(&mut db, list_id)
        .await?
        .ok_or(WebError::NotFound(Template::render("error/404", ())))?;

    Ok(Template::render("items/new", context! { list }))
}

#[post("/lists/<list_id>/items", format = "form", data = "<item>")]
pub async fn create(
    mut db: Connection<WishlistDb>,
    list_id: i64,
    item: Form<CreateItem<'_>>,
) -> Result<Redirect, WebError<Template>> {
    let list = List::find_by_id(&mut db, list_id)
        .await?
        .ok_or(WebError::NotFound(Template::render("error/404", ())))?;

    match Item::create(&mut db, list_id, item.title, item.description).await {
        Ok(item) => Ok(Redirect::to(uri!(web::items::show(list.id, item.id)))),
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

#[get("/lists/<list_id>/items/<id>", rank = 2)]
pub async fn show(
    mut db: Connection<WishlistDb>,
    list_id: i64,
    id: i64,
) -> Result<Template, WebError<Template>> {
    let list = List::find_by_id(&mut db, list_id)
        .await?
        .ok_or(WebError::NotFound(Template::render("error/404", ())))?;

    let item = Item::find_by_id(&mut db, id).await?;

    Ok(Template::render("items/show", context! { list, item }))
}

#[get("/lists/<list_id>/items/<id>/edit")]
pub async fn edit(
    mut db: Connection<WishlistDb>,
    list_id: i64,
    id: i64,
) -> Result<Template, WebError<Template>> {
    let list = List::find_by_id(&mut db, list_id)
        .await?
        .ok_or(WebError::NotFound(Template::render("error/404", ())))?;

    let item = Item::find_by_id(&mut db, id).await?;

    Ok(Template::render("items/edit", context! { list, item }))
}

#[put("/lists/<list_id>/items/<id>", format = "form", data = "<item>")]
pub async fn update(
    mut db: Connection<WishlistDb>,
    list_id: i64,
    id: i64,
    item: Form<EditItem<'_>>,
) -> Result<Redirect, WebError<Template>> {
    let list = List::find_by_id(&mut db, list_id)
        .await?
        .ok_or(WebError::NotFound(Template::render("error/404", ())))?;

    let mut old_item = Item::find_by_id(&mut db, id)
        .await?
        .ok_or(WebError::NotFound(Template::render("error/404", ())))?;

    match old_item
        .update(&mut db, &item.title, &item.description)
        .await
    {
        Ok(item) => Ok(Redirect::to(uri!(web::items::show(list.id, item.id)))),
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

#[delete("/lists/<list_id>/items/<id>")]
pub async fn destroy(
    mut db: Connection<WishlistDb>,
    list_id: i64,
    id: i64,
) -> Result<Redirect, WebError<Template>> {
    let list = List::find_by_id(&mut db, list_id)
        .await?
        .ok_or(WebError::NotFound(Template::render("error/404", ())))?;

    let mut item = Item::find_by_id(&mut db, id)
        .await?
        .ok_or(WebError::NotFound(Template::render("error/404", ())))?;

    item.destroy(&mut db).await?;

    Ok(Redirect::to(uri!(web::items::index(list.id))))
}
