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
    /// The title of the list.
    #[validate(length(min = 2, max = 256, message = "Title must be between 2 and 256 characters"))]
    pub title: String,
    /// A description of the list.
    #[validate(length(max = 4096, message = "Description must be less than 4096 characters"))]
    pub description: String,
}

impl Default for List {
    fn default() -> Self {
        Self {
            id: 0,
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
        title: &str,
        description: &str,
    ) -> Result<List, DataError> {
        List::new(title.to_string(), description.to_string()).save(conn).await
    }

    /// Creates a new list without saving it to the database.
    pub fn new(title: String, description: String) -> List {
        List {
            id: 0,
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

    /// Returns all lists in the database.
    pub async fn all(conn: &mut Connection<WishlistDb>) -> Result<Vec<List>, sqlx::Error> {
        sqlx::query_as(r#"SELECT id, title, description FROM lists"#)
            .fetch_all(&mut **conn)
            .await
    }

    /// Returns the list with the given ID, or `None` if no list with that ID exists.
    pub async fn find_by_id(
        conn: &mut Connection<WishlistDb>,
        id: i64,
    ) -> Result<Option<List>, sqlx::Error> {
        sqlx::query_as(r#"SELECT id, title, description FROM lists WHERE id = $1"#)
            .bind(id)
            .fetch_optional(&mut **conn)
            .await
    }

    /// Updates the list in the database, returning an updated copy of the list.
    pub async fn update(
        &mut self,
        conn: &mut Connection<WishlistDb>,
        title: &str,
        description: &str
    ) -> Result<List, DataError> {
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

    /// Deletes the list with the given ID from the database.
    pub async fn destroy_by_id(
        conn: &mut Connection<WishlistDb>,
        id: i64,
    ) -> Result<(), DataError> {
        List::do_delete(conn, id).await
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
            r#"INSERT INTO lists (title, description) VALUES ($1, $2) RETURNING id, title, description"#,
        )
        .bind(&self.title)
        .bind(&self.description)
        .fetch_one(&mut **conn)
        .await?;

        Ok(list)
    }

    async fn do_update(&self, conn: &mut Connection<WishlistDb>) -> Result<List, DataError> {
        self.validate()?;

        let list = sqlx::query_as(
            r#"UPDATE lists SET title = $1, description = $2 WHERE id = $3 RETURNING id, title, description"#,
        )
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
