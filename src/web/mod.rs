use rocket_dyn_templates::{context, Template};

use crate::db::DataError;

pub mod items;
pub mod lists;

#[derive(Responder)]
pub enum WebError<T> {
    #[response(status = 422)]
    Invalid(T),
    #[response(status = 404)]
    NotFound(T),
    #[response(status = 500)]
    Internal(T),
}

impl From<sqlx::Error> for WebError<String> {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::RowNotFound => WebError::NotFound(e.to_string()),
            _ => WebError::Internal(e.to_string()),
        }
    }
}

impl From<sqlx::Error> for WebError<Template> {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::RowNotFound => WebError::NotFound(Template::render("error/404", ())),
            _ => WebError::Internal(Template::render("error/500", ())),
        }
    }
}

impl From<DataError> for WebError<String> {
    fn from(e: DataError) -> Self {
        match e {
            DataError::Validation(e) => WebError::Invalid(e.to_string()),
            DataError::Sqlx(e) => e.into(),
        }
    }
}

impl From<DataError> for WebError<Template> {
    fn from(e: DataError) -> Self {
        match e {
            DataError::Validation(e) => WebError::Invalid(Template::render(
                "error/422",
                context! {
                    error_message: "Fix your errors",
                    errors: e,
                },
            )),
            DataError::Sqlx(e) => e.into(),
        }
    }
}
