use rocket::serde::{Deserialize, Serialize};
use rocket_db_pools::sqlx;
use rocket_db_pools::Connection;
use validator::Validate;

use crate::db::DataError;
use crate::db::WishlistDb;

/// A item of items.
#[derive(sqlx::FromRow, Debug, Validate, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Item {
    /// The item's unique ID.
    pub id: i64,
    /// The id of the list this item belongs to.
    #[validate(range(min = 1, message = "Invalid list ID"))]
    pub list_id: i64,
    /// The title of the item.
    #[validate(length(min = 2, max = 256, message = "Title must be between 2 and 256 characters"))]
    pub title: String,
    /// A description of the item.
    #[validate(length(max = 4096, message = "Description must be less than 4096 characters"))]
    pub description: String,
}

impl Default for Item {
    fn default() -> Self {
        Self {
            id: 0,
            list_id: 0,
            title: "".to_string(),
            description: "".to_string(),
        }
    }
}

impl Item {
    /// Shorthand for `item::new(...).save(conn)`.
    /// 
    /// Creates a new item and saves it to the database, returning the new item.
    pub async fn create(
        conn: &mut Connection<WishlistDb>,
        list_id: i64,
        title: &str,
        description: &str,
    ) -> Result<Item, DataError> {
        Item::new(list_id, title.to_string(), description.to_string()).save(conn).await
    }

    /// Creates a new item without saving it to the database.
    pub fn new(list_id: i64, title: String, description: String) -> Item {
        Item {
            id: 0,
            list_id,
            title,
            description,
        }
    }

    /// Saves the item to the database, returning an updated copy of the item.
    pub async fn save(self, conn: &mut Connection<WishlistDb>) -> Result<Item, DataError> {
        if self.id == 0 {
            self.do_insert(conn).await
        } else {
            self.do_update(conn).await
        }
    }

    /// Returns all items in the database.
    pub async fn all_by_list(conn: &mut Connection<WishlistDb>, list_id: i64) -> Result<Vec<Item>, sqlx::Error> {
        sqlx::query_as(r#"SELECT id, list_id, title, description FROM items WHERE list_id = $1"#)
            .bind(list_id)
            .fetch_all(&mut **conn)
            .await
    }

    /// Returns the item with the given ID, or `None` if no item with that ID exists.
    pub async fn find_by_id(
        conn: &mut Connection<WishlistDb>,
        id: i64,
    ) -> Result<Option<Item>, sqlx::Error> {
        sqlx::query_as(r#"SELECT id, list_id, title, description FROM items WHERE id = $1"#)
            .bind(id)
            .fetch_optional(&mut **conn)
            .await
    }

    /// Updates the item in the database, returning an updated copy of the item.
    pub async fn update(
        &mut self,
        conn: &mut Connection<WishlistDb>,
        title: &str,
        description: &str
    ) -> Result<Item, DataError> {
        self.title = title.to_string();
        self.description = description.to_string();
        self.do_update(conn).await
    }

    /// Deletes the item from the database.
    pub async fn destroy(&mut self, conn: &mut Connection<WishlistDb>) -> Result<(), DataError> {
        if self.id != 0 {
            Item::do_delete(conn, self.id).await?;
            self.id = 0;
        }
        Ok(())
    }

    // ----- Misc -----

    /// Returns the number of items in the database.
    pub async fn count(conn: &mut Connection<WishlistDb>) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar(r#"SELECT COUNT(*) FROM items"#)
            .fetch_one(&mut **conn)
            .await
    }

    // ----- Internal -----

    async fn do_insert(self, conn: &mut Connection<WishlistDb>) -> Result<Item, DataError> {
        self.validate()?;

        let item = sqlx::query_as(
            r#"INSERT INTO items (list_id, title, description) VALUES ($1, $2, $3) RETURNING id, list_id, title, description"#,
        )
        .bind(&self.list_id)
        .bind(&self.title)
        .bind(&self.description)
        .fetch_one(&mut **conn)
        .await?;

        Ok(item)
    }

    async fn do_update(&self, conn: &mut Connection<WishlistDb>) -> Result<Item, DataError> {
        self.validate()?;

        let item = sqlx::query_as(
            r#"UPDATE items SET list_id = $1,  title = $2, description = $3 WHERE id = $4 RETURNING id, list_id, title, description"#,
        )
        .bind(&self.list_id)
        .bind(&self.title)
        .bind(&self.description)
        .bind(self.id)
        .fetch_one(&mut **conn)
        .await?;

        Ok(item)
    }

    async fn do_delete(conn: &mut Connection<WishlistDb>, id: i64) -> Result<(), DataError> {
        sqlx::query(r#"DELETE FROM items WHERE id = $1"#)
            .bind(id)
            .execute(&mut **conn)
            .await?;
        Ok(())
    }
}
