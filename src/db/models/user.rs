use rocket::serde::{Deserialize, Serialize};
use rocket_db_pools::{sqlx, Connection};
use validator::Validate;

use crate::db::{DataError, WishlistDb};

/// A user
#[derive(sqlx::FromRow, Debug, Validate, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String, 
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl User {
    /// Shorthand for `User::new(...).save(conn)`.
    ///
    /// Creates a new user and saves it to the database, returning the new user.
    pub async fn create(
        conn: &mut Connection<WishlistDb>,
        username: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<User, DataError> {
        User::new(
            username.to_string(),
            email.to_string(),
            password_hash.to_string(),
        )
        .save(conn)
        .await
    }

    /// Creates a new user without saving it to the database.
    pub fn new(username: String, email: String, password_hash: String) -> User {
        User {
            id: 0,
            username,
            email,
            password_hash,
            created_at: chrono::NaiveDateTime::default(),
            updated_at: chrono::NaiveDateTime::default(),
        }
    }

    /// Saves the user to the database, returning an updated copy of the user.
    pub async fn save(self, conn: &mut Connection<WishlistDb>) -> Result<User, DataError> {
        if self.id == 0 {
            self.do_insert(conn).await
        } else {
            self.do_update(conn).await
        }
    }

    /// Returns all users in the database.
    pub async fn all(conn: &mut Connection<WishlistDb>) -> Result<Vec<User>, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT id, username, email, '' as password_hash, created_at, updated_at
            FROM users
            "#,
        )
        .fetch_all(&mut **conn)
        .await
    }

    /// Returns the user with the given id, or `None` if no user with that id exists.
    pub async fn find_by_id(
        conn: &mut Connection<WishlistDb>,
        id: i64,
    ) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT id, username, email, password_hash, created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&mut **conn)
        .await
    }

    /// Returns the user with the given username, or `None` if no user with that username exists.
    pub async fn find_by_username(
        conn: &mut Connection<WishlistDb>,
        username: &str,
    ) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT id, username, email, password_hash, created_at, updated_at
            FROM users
            WHERE username = $1
            "#,
        )
        .bind(username)
        .fetch_optional(&mut **conn)
        .await
    }

    /// Updates the user in the database, returning an updated copy of the user.
    pub async fn update(
        &mut self,
        conn: &mut Connection<WishlistDb>,
        username: &str,
        email: &str,
    ) -> Result<User, DataError> {
        self.username = username.to_string();
        self.email = email.to_string();
        self.do_update(conn).await
    }

    /// Deletes the user from the database.
    pub async fn destroy(&mut self, conn: &mut Connection<WishlistDb>) -> Result<(), DataError> {
        if self.id != 0 {
            User::do_delete(conn, self.id).await?;
            self.id = 0;
        }
        Ok(())
    }

    // ----- Misc -----

    /// Returns the number of users in the database.
    pub async fn count(conn: &mut Connection<WishlistDb>) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar(r#"SELECT COUNT(*) FROM users"#)
            .fetch_one(&mut **conn)
            .await
    }

    // ----- Internal -----

    async fn do_insert(self, conn: &mut Connection<WishlistDb>) -> Result<User, DataError> {
        self.validate()?;

        let list = sqlx::query_as(
            r#"
            INSERT INTO users (username, email, password_hash, created_at, updated_at)
            VALUES ($1, $2, $3, now(), now())
            RETURNING id, username, email, '' as password_hash, created_at, updated_at
            "#,
        )
        .bind(&self.username)
        .bind(&self.email)
        .bind(&self.password_hash)
        .fetch_one(&mut **conn)
        .await?;

        Ok(list)
    }

    async fn do_update(&self, conn: &mut Connection<WishlistDb>) -> Result<User, DataError> {
        self.validate()?;

        let list = sqlx::query_as(
            r#"
            UPDATE users
            SET username = $1,
                email = $2,
                updated_at = now()
            WHERE id = $3
            RETURNING id, username, email, '' as password_hash, created_at, updated_at
            "#,
        )
        .bind(&self.username)
        .bind(&self.email)
        .bind(self.id)
        .fetch_one(&mut **conn)
        .await?;

        Ok(list)
    }

    async fn do_delete(conn: &mut Connection<WishlistDb>, id: i64) -> Result<(), DataError> {
        sqlx::query(r#"DELETE FROM users WHERE id = $1"#)
            .bind(id)
            .execute(&mut **conn)
            .await?;
        Ok(())
    }
}
