use rocket::form::Form;
use rocket::response::Redirect;
use rocket_db_pools::Connection;
use rocket_dyn_templates::{context, Template};

use crate::api::v1::lists::{CreateList, EditList};
use crate::db::models::{Item, List};
use crate::db::{DataError, WishlistDb};
use crate::web::{self, WebError};

#[get("/lists")]
pub async fn index(mut db: Connection<WishlistDb>) -> Result<Template, WebError<Template>> {
    let lists = List::all_public(&mut db)
        .await?
        .into_iter()
        .map(|list| {
            let link = uri!(show(&list.key)).to_string();
            context! {
                id: list.id,
                key: list.key,
                title: list.title,
                description: list.description,
                link,
            }
        })
        .collect::<Vec<_>>();

    Ok(Template::render("lists/index", context! { lists: lists }))
}

#[get("/lists/new")]
pub fn new() -> Template {
    Template::render("lists/new", context! { list: List::default() })
}

#[post("/lists", format = "form", data = "<list>")]
pub async fn create(
    mut db: Connection<WishlistDb>,
    list: Form<CreateList<'_>>,
) -> Result<Redirect, WebError<Template>> {
    match List::create(&mut db, list.is_private, list.title, list.description).await {
        Ok(list) => Ok(Redirect::to(uri!(web::lists::show(list.key)))),
        Err(DataError::Validation(e)) => Err(WebError::Invalid(Template::render(
            "lists/new",
            context! {
                list: context! {
                    is_private: list.is_private,
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
                    is_private: list.is_private,
                    title: list.title,
                    description: list.description,
                },
                error_message: e.to_string()
            },
        ))),
    }
}

#[get("/lists/<key>")]
pub async fn show(
    mut db: Connection<WishlistDb>,
    key: &str,
) -> Result<Template, WebError<Template>> {
    let list = List::find_by_key(&mut db, key)
        .await?
        .ok_or(WebError::NotFound(Template::render("error/404", ())))?;

    let items = Item::all_by_list(&mut db, list.id).await?;

    Ok(Template::render("lists/show", context! { list, items }))
}

#[get("/lists/<key>/edit")]
pub async fn edit(
    mut db: Connection<WishlistDb>,
    key: &str,
) -> Result<Template, WebError<Template>> {
    let list = List::find_by_key(&mut db, key)
        .await?
        .ok_or(WebError::NotFound(Template::render("error/404", ())))?;

    Ok(Template::render("lists/edit", context! { list }))
}

#[put("/lists/<key>", format = "form", data = "<list>")]
pub async fn update(
    mut db: Connection<WishlistDb>,
    key: &str,
    list: Form<EditList<'_>>,
) -> Result<Redirect, WebError<Template>> {
    let mut old_list = List::find_by_key(&mut db, key)
        .await?
        .ok_or(WebError::NotFound(Template::render("error/404", ())))?;

    match old_list
        .update(&mut db, list.is_private, &list.title, &list.description)
        .await
    {
        Ok(list) => Ok(Redirect::to(uri!(web::lists::show(list.key)))),
        Err(DataError::Validation(e)) => Err(WebError::Invalid(Template::render(
            "lists/edit",
            context! {
               list: context! {
                    key,
                    is_private: list.is_private,
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
                    key,
                    is_private: list.is_private,
                    title: list.title,
                    description: list.description,
                },
                error_message: e.to_string()
            },
        ))),
    }
}

#[delete("/lists/<key>")]
pub async fn destroy(
    mut db: Connection<WishlistDb>,
    key: &str,
) -> Result<Redirect, WebError<Template>> {
    let mut list = List::find_by_key(&mut db, key)
        .await?
        .ok_or(WebError::NotFound(Template::render("error/404", ())))?;

    list.destroy(&mut db).await?;

    Ok(Redirect::to(uri!(web::lists::index)))
}
