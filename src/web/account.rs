use rocket::form::Form;
use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket_db_pools::Connection;
use rocket_dyn_templates::{context, Template};

use crate::db::{DataError, WishlistDb};
use crate::web::auth::{self, NewUser, UserLogin};
use crate::web::WebError;

use super::auth::LoggedInUser;

#[get("/account")]
pub fn show(user: &'_ LoggedInUser) -> Template {
    Template::render("account/index", context! { user })
}

#[get("/account", rank = 2)]
pub fn show_2() -> Redirect {
    Redirect::to(uri!(login))
}

#[get("/account/register")]
pub fn new(_user: &'_ LoggedInUser) -> Redirect {
    Redirect::to(uri!(crate::web_index))
}

#[get("/account/register", rank = 2)]
pub fn new_2() -> Template {
    Template::render("account/register", context! {})
}

#[post("/account/register")]
pub async fn create(_user: &'_ LoggedInUser) -> Redirect {
    Redirect::to(uri!(crate::web_index))
}

#[post("/account/register", format = "form", data = "<user>", rank = 2)]
pub async fn create_2(
    mut db: Connection<WishlistDb>,
    user: Form<NewUser<'_>>,
) -> Result<Redirect, WebError<Template>> {
    let user = user.into_inner();
    match auth::register_new_user(&mut db, &user).await {
        Ok(_) => Ok(Redirect::to(uri!(crate::web_index))),
        Err(DataError::Validation(e)) => Err(WebError::Invalid(Template::render(
            "account/register",
            context! {
                register: context! {
                    username: user.username,
                    email: user.email,
                    password: user.password,
                    password_confirm: user.password_confirm,
                },
                error_message: "Fix your errors",
                errors: e,
            },
        ))),
        Err(e) => Err(WebError::Invalid(Template::render(
            "account/register",
            context! {
                register: context! {
                    username: user.username,
                    email: user.email,
                    password: user.password,
                    password_confirm: user.password_confirm,
                },
                error_message: e.to_string()
            },
        ))),
    }
}

#[get("/login")]
pub fn login(_user: &'_ LoggedInUser) -> Redirect {
    Redirect::to(uri!(crate::web_index))
}

#[get("/login", rank = 2)]
pub fn login_2() -> Template {
    Template::render("account/login", context! {})
}

#[post("/login")]
pub fn do_login(_user: &'_ LoggedInUser) -> Redirect {
    Redirect::to(uri!(crate::web_index))
}

#[post("/login", format = "form", data = "<login>", rank = 2)]
pub async fn do_login_2(
    mut db: Connection<WishlistDb>,
    cookies: &CookieJar<'_>,
    login: Form<UserLogin<'_>>,
) -> Result<Redirect, WebError<Template>> {
    // TODO: Redirect user if they're already logged in
    let login = login.into_inner();
    match auth::verify_user_login(&mut db, &login).await {
        Ok(user) => {
            auth::create_user_session(&mut db, cookies, &user).await?;
            Ok(Redirect::to(uri!(crate::web_index)))
        },
        Err(e) => Err(WebError::Invalid(Template::render(
            "account/login",
            context! {
                login: context! {
                    username: login.username,
                    password: login.password,
                },
                error_message: e.to_string()
            },
        ))),
    }
}

#[post("/logout")]
pub async fn logout(
    mut db: Connection<WishlistDb>,
    cookies: &CookieJar<'_>,
    _user: &'_ LoggedInUser,
) -> Result<Redirect, WebError<Template>> {
    auth::destroy_user_session(&mut db, cookies).await?;

    Ok(Redirect::to(uri!(crate::web_index)))
}

#[post("/logout", rank = 2)]
pub async fn logout_2() -> Result<Redirect, WebError<Template>> {
    Ok(Redirect::to(uri!(crate::web_index)))
}
