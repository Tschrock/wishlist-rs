use rocket::form::Form;
use rocket::response::Redirect;
use rocket_db_pools::Connection;
use rocket_dyn_templates::{context, Template};

use crate::api::v1::lists::{CreateList, EditList};
use crate::db::models::{Item, List};
use crate::db::{DataError, WishlistDb};
use crate::web::{self, WebError};

#[get("/lists")]
pub async fn index(mut db: Connection<WishlistDb>) -> Template {
    let lists = List::all(&mut db)
        .await
        .unwrap_or(vec![])
        .into_iter()
        .map(|list| context! {
            id: list.id,
            title: list.title,
            description: list.description,
            link: uri!(show(list.id)).to_string(),
        })
        .collect::<Vec<_>>();
    Template::render("lists/index", context! { lists: lists })
}

#[get("/lists/new")]
pub fn new() -> Template {
    Template::render("lists/new", ())
}

#[post("/lists", format = "form", data = "<list>")]
pub async fn create(
    mut db: Connection<WishlistDb>,
    list: Form<CreateList<'_>>,
) -> Result<Redirect, WebError<Template>> {
    match List::create(&mut db, list.title, list.description).await {
        Ok(list) => Ok(Redirect::to(uri!(web::lists::show(list.id)))),
        Err(DataError::Validation(e)) => Err(WebError::Invalid(Template::render(
            "lists/new",
            context! {
                list: context! {
                    title: list.title,
                    description: list.description,
                },
                error_message: "Fix your errors",
                errors: e,
            },
        ))),
        Err(e) => Err(WebError::Invalid(Template::render(
            "lists/new",
            context! {
                list: context! {
                    title: list.title,
                    description: list.description,
                },
                error_message: e.to_string()
            },
        ))),
    }
}

#[get("/lists/<id>")]
pub async fn show(mut db: Connection<WishlistDb>, id: i64) -> Result<Template, WebError<String>> {
    let list = List::find_by_id(&mut db, id).await?;
    let items = Item::all_by_list(&mut db, id).await?;

    Ok(Template::render("lists/show", context! { list, items }))
}

#[get("/lists/<id>/edit")]
pub async fn edit(mut db: Connection<WishlistDb>, id: i64) -> Result<Template, WebError<String>> {
    let list = List::find_by_id(&mut db, id).await?;

    Ok(Template::render("lists/edit", context! { list }))
}

#[put("/lists/<id>", format = "form", data = "<list>")]
pub async fn update(
    mut db: Connection<WishlistDb>,
    id: i64,
    list: Form<EditList<'_>>,
) -> Result<Redirect, WebError<Template>> {
    let mut old_list = List::find_by_id(&mut db, id)
        .await?
        .ok_or(WebError::NotFound(Template::render("error/404", ())))?;

    match old_list
        .update(&mut db, &list.title, &list.description)
        .await
    {
        Ok(list) => Ok(Redirect::to(uri!(web::lists::show(list.id)))),
        Err(DataError::Validation(e)) => Err(WebError::Invalid(Template::render(
            "lists/edit",
            context! {
               list: context! {
                    id,
                    title: list.title,
                    description: list.description,
               },
               error_message: "Fix your errors",
               errors: e,
            },
        ))),
        Err(e) => Err(WebError::Invalid(Template::render(
            "lists/edit",
            context! {
                list: context! {
                    id,
                    title: list.title,
                    description: list.description,
                },
                error_message: e.to_string()
            },
        ))),
    }
}

#[delete("/lists/<id>")]
pub async fn destroy(mut db: Connection<WishlistDb>, id: i64) -> Result<Redirect, WebError<Template>> {
    let mut list = List::find_by_id(&mut db, id)
        .await?
        .ok_or(WebError::NotFound(Template::render("error/404", ())))?;

    list.destroy(&mut db).await?;

    Ok(Redirect::to(uri!(web::lists::index)))
}
