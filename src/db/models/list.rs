use rocket::serde::{Deserialize, Serialize};
use rocket_db_pools::sqlx;
use rocket_db_pools::Connection;
use validator::Validate;

use crate::db::DataError;
use crate::db::WishlistDb;

/// A list of items.
#[derive(sqlx::FromRow, Debug, Validate, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct List {
    /// The list's unique ID.
    pub id: i64,
    /// The list's url key.
    pub key: String,
    /// Whether the list is private.
    pub is_private: bool,
    /// The title of the list.
    #[validate(length(
        min = 2,
        max = 256,
        message = "Title must be between 2 and 256 characters"
    ))]
    pub title: String,
    /// A description of the list.
    #[validate(length(max = 4096, message = "Description must be less than 4096 characters"))]
    pub description: String,
}

impl Default for List {
    fn default() -> Self {
        Self {
            id: 0,
            key: crate::util::random_key(),
            is_private: true,
            title: "".to_string(),
            description: "".to_string(),
        }
    }
}

impl List {
    /// Shorthand for `List::new(...).save(conn)`.
    ///
    /// Creates a new list and saves it to the database, returning the new list.
    pub async fn create(
        conn: &mut Connection<WishlistDb>,
        is_private: bool,
        title: &str,
        description: &str,
    ) -> Result<List, DataError> {
        List::new(is_private, title.to_string(), description.to_string())
            .save(conn)
            .await
    }

    /// Creates a new list without saving it to the database.
    pub fn new(is_private: bool, title: String, description: String) -> List {
        List {
            id: 0,
            key: crate::util::random_key(),
            is_private,
            title,
            description,
        }
    }

    /// Saves the list to the database, returning an updated copy of the list.
    pub async fn save(self, conn: &mut Connection<WishlistDb>) -> Result<List, DataError> {
        if self.id == 0 {
            self.do_insert(conn).await
        } else {
            self.do_update(conn).await
        }
    }

    /// Returns all public lists in the database.
    pub async fn all_public(conn: &mut Connection<WishlistDb>) -> Result<Vec<List>, sqlx::Error> {
        sqlx::query_as(r#"SELECT id, key, is_private, title, description FROM lists WHERE is_private IS FALSE"#)
            .fetch_all(&mut **conn)
            .await
    }

    /// Returns the list with the given Key, or `None` if no list with that Key exists.
    pub async fn find_by_key(
        conn: &mut Connection<WishlistDb>,
        key: &str,
    ) -> Result<Option<List>, sqlx::Error> {
        sqlx::query_as(r#"SELECT id, key, is_private, title, description FROM lists WHERE key = $1"#)
            .bind(key)
            .fetch_optional(&mut **conn)
            .await
    }

    /// Updates the list in the database, returning an updated copy of the list.
    pub async fn update(
        &mut self,
        conn: &mut Connection<WishlistDb>,
        is_private: bool,
        title: &str,
        description: &str,
    ) -> Result<List, DataError> {
        self.is_private = is_private;
        self.title = title.to_string();
        self.description = description.to_string();
        self.do_update(conn).await
    }

    /// Deletes the list from the database.
    pub async fn destroy(&mut self, conn: &mut Connection<WishlistDb>) -> Result<(), DataError> {
        if self.id != 0 {
            List::do_delete(conn, self.id).await?;
            self.id = 0;
        }
        Ok(())
    }

    // ----- Misc -----

    /// Returns the number of lists in the database.
    pub async fn count(conn: &mut Connection<WishlistDb>) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar(r#"SELECT COUNT(*) FROM lists"#)
            .fetch_one(&mut **conn)
            .await
    }

    // ----- Internal -----

    async fn do_insert(self, conn: &mut Connection<WishlistDb>) -> Result<List, DataError> {
        self.validate()?;

        let list = sqlx::query_as(
            r#"INSERT INTO lists (key, is_private, title, description) VALUES ($1, $2, $3, $4) RETURNING id, key, is_private, title, description"#,
        )
        .bind(&self.key)
        .bind(&self.is_private)
        .bind(&self.title)
        .bind(&self.description)
        .fetch_one(&mut **conn)
        .await?;

        Ok(list)
    }

    async fn do_update(&self, conn: &mut Connection<WishlistDb>) -> Result<List, DataError> {
        self.validate()?;

        let list = sqlx::query_as(
            r#"UPDATE lists SET is_private = $1, title = $2, description = $3 WHERE id = $4 RETURNING id, key, is_private, title, description"#,
        )
        .bind(&self.is_private)
        .bind(&self.title)
        .bind(&self.description)
        .bind(self.id)
        .fetch_one(&mut **conn)
        .await?;

        Ok(list)
    }

    async fn do_delete(conn: &mut Connection<WishlistDb>, id: i64) -> Result<(), DataError> {
        sqlx::query(r#"DELETE FROM lists WHERE id = $1"#)
            .bind(id)
            .execute(&mut **conn)
            .await?;
        Ok(())
    }
}
