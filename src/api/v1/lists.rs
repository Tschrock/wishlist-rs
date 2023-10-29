use rocket::http::Status;
use rocket::response::status::{self, Created, NoContent};
use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use rocket_db_pools::Connection;

use crate::api::ApiError;
use crate::db::models::List;
use crate::db::WishlistDb;

#[derive(FromForm, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct CreateList<'r> {
    pub is_private: bool,
    pub title: &'r str,
    pub description: &'r str,
}

#[derive(FromForm, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct EditList<'r> {
    pub is_private: bool,
    pub title: &'r str,
    pub description: &'r str,
}

#[get("/api/v1/lists")]
pub async fn index(mut db: Connection<WishlistDb>) -> Result<Json<Vec<List>>, ApiError> {
    let list = List::all_public(&mut db).await?;

    Ok(Json(list))
}

#[get("/api/v1/lists/<key>")]
pub async fn show(
    mut db: Connection<WishlistDb>,
    key: &str,
) -> Result<Option<Json<List>>, ApiError> {
    let list = List::find_by_key(&mut db, key).await?;

    Ok(list.map(Json))
}

#[post("/api/v1/lists", data = "<list>")]
pub async fn create(
    mut db: Connection<WishlistDb>,
    list: Json<CreateList<'_>>,
) -> Result<Created<Json<List>>, status::Custom<String>> {
    List::create(&mut db, list.is_private, list.title, list.description)
        .await
        .map(|new_list| Created::new(uri!(show(&new_list.key)).to_string()).body(Json(new_list)))
        .map_err(|e| status::Custom(Status::InternalServerError, e.to_string()))
}

#[put("/api/v1/lists/<key>", data = "<list>")]
pub async fn update(
    mut db: Connection<WishlistDb>,
    key: &str,
    list: Json<EditList<'_>>,
) -> Result<Json<List>, ApiError> {
    let mut old_list = List::find_by_key(&mut db, key)
        .await?
        .ok_or(ApiError::NotFound(Json(crate::api::ApiGenericError {
            message: "List not found".to_string(),
        })))?;

    let new_list = old_list
        .update(&mut db, list.is_private, list.title, list.description)
        .await?;

    Ok(Json(new_list))
}

#[delete("/api/v1/lists/<key>")]
pub async fn destroy(mut db: Connection<WishlistDb>, key: &str) -> Result<NoContent, ApiError> {
    let mut list = List::find_by_key(&mut db, key)
        .await?
        .ok_or(ApiError::NotFound(Json(crate::api::ApiGenericError {
            message: "List not found".to_string(),
        })))?;

    list.destroy(&mut db).await?;

    Ok(NoContent)
}
