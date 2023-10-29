use std::borrow::Cow;

use bcrypt::BcryptError;
use rocket::http::{Cookie, CookieJar};
use rocket::outcome::IntoOutcome;
use rocket::request::{FromRequest, Outcome, Request};
use rocket::serde::{Deserialize, Serialize};
use rocket::time::Duration;
use rocket_db_pools::Connection;
use thiserror::Error;
use validator::Validate;

use crate::db::models::{User, UserSession};
use crate::db::{DataError, WishlistDb};

#[derive(FromForm, Validate, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct NewUser<'r> {
    // validation happens in the user model
    pub username: &'r str,
    // validation happens in the user model
    pub email: &'r str,
    #[validate(
        length(
            min = 8,
            max = 128,
            message = "Password must be longer than 8 characters."
        ),
        custom = "validate_password"
    )]
    pub password: &'r str,
    #[validate(must_match(other = "password", message = "Passwords must match"))]
    pub password_confirm: &'r str,
}

#[derive(FromForm, Validate, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct UserLogin<'r> {
    pub username: &'r str,
    pub password: &'r str,
}

impl From<BcryptError> for DataError {
    fn from(e: BcryptError) -> Self {
        DataError::Other(e.to_string())
    }
}

pub fn validate_password(password: &str) -> Result<(), validator::ValidationError> {
    if password == "password" {
        let mut err = validator::ValidationError::new("insecure");
        err.message = Some(Cow::from("Password cannot be 'password'".to_string()));
        Err(err)
    } else if password == "hunter2" {
        let mut err = validator::ValidationError::new("insecure");
        err.message = Some(Cow::from("Nice try, but no.".to_string()));
        Err(err)
    } else {
        Ok(())
    }
}

pub async fn register_new_user(
    conn: &mut Connection<WishlistDb>,
    user: &NewUser<'_>,
) -> Result<User, DataError> {
    // Validate the new user form
    user.validate()?;

    // Hash password
    let password_hash = bcrypt::hash(user.password, bcrypt::DEFAULT_COST)?;

    // Create the new user
    // The db layer should handle the uniqueness constraint
    let user = User::create(conn, user.username, user.email, &password_hash).await?;

    Ok(user)
}

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Incorrect username or password")]
    InvalidLogin,
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Bcrypt error: {0}")]
    Bcrypt(#[from] bcrypt::BcryptError),
    // #[error("Unknown error: {0}")]
    // Unknown(String),
}

pub async fn verify_user_login(
    conn: &mut Connection<WishlistDb>,
    login: &UserLogin<'_>,
) -> Result<User, AuthError> {
    // Get the user from the database
    let user = User::find_by_username(conn, login.username)
        .await?
        .ok_or(AuthError::InvalidLogin)?;

    // Verify the password
    if bcrypt::verify(login.password, &user.password_hash)? {
        Ok(user)
    } else {
        Err(AuthError::InvalidLogin)
    }
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct LoggedInUser {
    pub user: User,
}

impl LoggedInUser {
    pub fn new(user: User) -> Self {
        Self { user }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for &'r LoggedInUser {
    type Error = ();
    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let user_result = request
            .local_cache_async(async {
                // Get the session cookie
                let session_cookie = request.cookies().get("session_id")?;

                // Get the session token
                let session_token = session_cookie.value();

                // Get the database connection
                let mut db = request
                    .guard::<Connection<WishlistDb>>()
                    .await
                    .succeeded()?;

                // Get the user session from the database
                let user_session = UserSession::find_by_token(&mut db, session_token)
                    .await
                    .ok()??;

                // Get the user from the database
                User::find_by_id(&mut db, user_session.user_id)
                    .await
                    .ok()?
                    .map(|u| LoggedInUser::new(u))
            })
            .await;

        user_result.as_ref().or_forward(())
    }
}

pub async fn create_user_session(
    conn: &mut Connection<WishlistDb>,
    cookies: &CookieJar<'_>,
    user: &User,
) -> Result<UserSession, DataError> {
    // Generate a new session token
    let session_token = crate::util::random_token();

    // Create the new session
    let session = UserSession::create(conn, &session_token, user.id).await?;

    // Set the session cookie
    let cookie = Cookie::build("session_id", session_token)
        .path("/")
        .http_only(true)
        .max_age(Duration::days(7))
        .same_site(rocket::http::SameSite::Strict)
        .finish();

    cookies.add(cookie);

    Ok(session)
}

pub async fn destroy_user_session(
    conn: &mut Connection<WishlistDb>,
    cookies: &CookieJar<'_>,
) -> Result<(), DataError> {
    // Get the session cookie
    let session_cookie = match cookies.get("session_id") {
        Some(cookie) => cookie,
        None => return Ok(()),
    };

    // Get the session token
    let session_token = session_cookie.value();

    // Delete the session from the database
    UserSession::destroy_by_token(conn, session_token).await?;

    // Delete the session cookie
    cookies.remove(session_cookie.clone());

    Ok(())
}
