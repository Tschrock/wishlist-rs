use rocket::serde::{Deserialize, Serialize};
use rocket_db_pools::sqlx;
use validator::Validate;


/// An image
#[derive(sqlx::FromRow, Debug, Validate, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Image {
    /// The image's unique ID.
    pub id: i64,

    /// If the image was fetched from an external source, the URL of that source.
    pub source_url: Option<String>,
}
