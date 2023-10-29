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
    pub title: &'r str,
    pub description: &'r str,
}

#[derive(FromForm, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct EditList<'r> {
    pub title: &'r str,
    pub description: &'r str,
}

#[get("/api/v1/lists")]
pub async fn index(mut db: Connection<WishlistDb>) -> Result<Json<Vec<List>>, ApiError> {
    let list = List::all(&mut db).await?;

    Ok(Json(list))
}

#[get("/api/v1/lists/<id>")]
pub async fn show(mut db: Connection<WishlistDb>, id: i64) -> Result<Option<Json<List>>, ApiError> {
    let list = List::find_by_id(&mut db, id).await?;

    Ok(list.map(Json))
}

#[post("/api/v1/lists", data = "<list>")]
pub async fn create(
    mut db: Connection<WishlistDb>,
    list: Json<CreateList<'_>>,
) -> Result<Created<Json<List>>, status::Custom<String>> {
    List::create(&mut db, list.title, list.description)
        .await
        .map(|res| Created::new(uri!(show(res.id)).to_string()).body(Json(res)))
        .map_err(|e| status::Custom(Status::InternalServerError, e.to_string()))
}

#[put("/api/v1/lists/<id>", data = "<list>")]
pub async fn update(
    mut db: Connection<WishlistDb>,
    id: i64,
    list: Json<EditList<'_>>,
) -> Result<Json<List>, ApiError> {
    let mut old_list = List::find_by_id(&mut db, id)
        .await?
        .ok_or(ApiError::NotFound(Json(crate::api::ApiGenericError {
            message: "List not found".to_string(),
        })))?;

    let new_list = old_list
        .update(&mut db, list.title, list.description)
        .await?;

    Ok(Json(new_list))
}

#[delete("/api/v1/lists/<id>")]
pub async fn destroy(mut db: Connection<WishlistDb>, id: i64) -> Result<NoContent, ApiError> {
    List::destroy_by_id(&mut db, id).await?;

    Ok(NoContent)
}
