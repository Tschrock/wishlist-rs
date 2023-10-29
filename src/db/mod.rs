use rocket::{fairing, Build, Rocket};
use rocket_db_pools::{sqlx, Database};
use thiserror::Error;
use validator::ValidationErrors;

pub mod models;

/// The database connection pool.
#[derive(Database)]
#[database("wishlists")]
pub struct WishlistDb(sqlx::AnyPool);

static DB_URL_CONFIG_KEY: &str = "databases.wishlists.url";

/// Handles default database initialization.
pub async fn default_db(rocket: Rocket<Build>) -> fairing::Result {
    // If the database is sqlite, make sure the file exists
    if let Ok(url) = rocket.figment().extract_inner::<String>(DB_URL_CONFIG_KEY) {
        if url.starts_with("sqlite:") {
            let path = std::path::Path::new(url.trim_start_matches("sqlite:"));
            match crate::util::ensure_file_exists(path, None) {
                Ok(_) => (),
                Err(e) => {
                    error!("SQLite database could not be opened: {}", e);
                    return Err(rocket);
                }
            }
        }
    }

    // Return the rocket instance
    Ok(rocket)
}

/// Runs the database migrations.
pub async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
    match WishlistDb::fetch(&rocket) {
        Some(db) => {
            let mig = match db.any_kind() {
                sqlx::any::AnyKind::Postgres => sqlx::migrate!("./migrations/postgres"),
                sqlx::any::AnyKind::Sqlite => sqlx::migrate!("./migrations/sqlite"),
            };
            match mig.run(&**db).await {
                Ok(_) => Ok(rocket),
                Err(e) => {
                    error!("Failed to initialize SQLx database: {}", e);
                    Err(rocket)
                }
            }
        }
        None => Err(rocket),
    }
}

#[derive(Error, Debug)]
pub enum DataError {
    #[error("Invalid data: {0}")]
    Validation(#[from] ValidationErrors),
    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("Data error: {0}")]
    Other(String),
}
