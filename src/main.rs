#[macro_use]
extern crate rocket;

use rocket::fairing::AdHoc;
use rocket_db_pools::Connection;
use rocket_db_pools::Database;
use rocket_dyn_templates::{context, Template};

mod api;
mod db;
mod web;

use db::models::{Item, List};
use db::WishlistDb;

//--------------------
// Web Pages
//--------------------

#[get("/")]
async fn web_index(mut db: Connection<WishlistDb>) -> Template {
    Template::render(
        "index",
        context! {
            list_count: List::count(&mut db).await.unwrap_or(0),
            item_count: Item::count(&mut db).await.unwrap_or(0)
        },
    )
}

#[launch]
fn rocket() -> _ {
    let figment = rocket::Config::figment();

    rocket::custom(figment)
        .attach(AdHoc::try_on_ignite("Default DB", db::default_db))
        .attach(WishlistDb::init())
        .attach(AdHoc::try_on_ignite("Migrations", db::run_migrations))
        .attach(Template::fairing())
        .mount(
            "/",
            routes![
                // Web Misc
                web_index,
                // Web Lists
                web::lists::index,
                web::lists::new,
                web::lists::create,
                web::lists::show,
                web::lists::edit,
                web::lists::update,
                web::lists::destroy,
                // Web Items
                web::items::index,
                web::items::new,
                web::items::create,
                web::items::show,
                web::items::edit,
                web::items::update,
                web::items::destroy,
                // API Lists
                api::v1::lists::index,
                api::v1::lists::create,
                api::v1::lists::show,
                api::v1::lists::update,
                api::v1::lists::destroy,
            ],
        )
}
