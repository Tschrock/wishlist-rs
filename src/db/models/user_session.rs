use chrono::{Days, Utc};
use rocket::serde::{Deserialize, Serialize};
use rocket_db_pools::{sqlx, Connection};

use crate::db::{DataError, WishlistDb};

/// A user session
#[derive(sqlx::FromRow, Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct UserSession {
    pub id: i64,
    pub token: String,
    pub user_id: i64,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl UserSession {
    pub async fn create(
        conn: &mut Connection<WishlistDb>,
        token: &str,
        user_id: i64,
    ) -> Result<UserSession, DataError> {
        let user_session = sqlx::query_as(
            r#"
            INSERT INTO user_sessions (token, user_id, created_at, updated_at)
            VALUES ($1, $2, now(), now())
            RETURNING id, token, user_id, created_at, updated_at
            "#,
        )
        .bind(token)
        .bind(user_id)
        .fetch_one(&mut **conn)
        .await?;

        Ok(user_session)
    }

    pub async fn find_by_token(
        conn: &mut Connection<WishlistDb>,
        token: &str,
    ) -> Result<Option<UserSession>, DataError> {
        let session = sqlx::query_as(r#"SELECT id, token, user_id, created_at, updated_at FROM user_sessions WHERE token = $1"#)
            .bind(token)
            .fetch_optional(&mut **conn)
            .await?;

        Ok(session)
    }

    pub async fn destroy_by_token(
        conn: &mut Connection<WishlistDb>,
        token: &str,
    ) -> Result<(), DataError> {
        sqlx::query(r#"DELETE FROM user_sessions WHERE token = $1"#)
            .bind(token)
            .execute(&mut **conn)
            .await?;
        Ok(())
    }

    pub async fn destroy_outdated(conn: &mut Connection<WishlistDb>) -> Result<(), DataError> {
        let remove_before = Utc::now().checked_sub_days(Days::new(7)).unwrap();
        sqlx::query(r#"DELETE FROM user_sessions WHERE created_at < $1"#)
            .bind(remove_before)
            .execute(&mut **conn)
            .await?;
        Ok(())
    }
}
