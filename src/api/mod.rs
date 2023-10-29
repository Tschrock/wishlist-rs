use rocket::serde::json::Json;
use rocket::serde::Serialize;
use validator::ValidationErrors;

use crate::db::DataError;

pub mod v1;

#[derive(Responder)]
pub enum ApiError {
    // #[response(status = 400)]
    // Unauthorized(Json<ApiGenericError>),
    #[response(status = 422)]
    Invalid(Json<ValidationErrors>),
    #[response(status = 404)]
    NotFound(Json<ApiGenericError>),
    #[response(status = 500)]
    Internal(Json<ApiGenericError>),
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct ApiGenericError {
    pub message: String,
}

impl From<sqlx::Error> for ApiError {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::RowNotFound => ApiError::NotFound(Json(ApiGenericError {
                message: e.to_string(),
            })),
            _ => ApiError::Internal(Json(ApiGenericError {
                message: e.to_string(),
            })),
        }
    }
}

impl From<DataError> for ApiError {
    fn from(e: DataError) -> Self {
        match e {
            DataError::Validation(e) => ApiError::Invalid(Json(e)),
            DataError::Sqlx(e) => e.into(),
            DataError::Other(e) => ApiError::Internal(Json(ApiGenericError {
                message: e.to_string(),
            })),
        }
    }
}
