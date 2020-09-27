use crate::schema::books;
use diesel::sqlite::SqliteConnection;
use async_diesel::*;
use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use r2d2::Pool;
use contraband::Injectable;
use contraband::graph::Value;

#[derive(Deserialize, Serialize, Queryable)]
pub struct Book {
    pub id: i32,
    pub title: String,
    pub author: String,
}

#[derive(Insertable)]
#[table_name = "books"]
pub struct InsertBook {
    pub title: String,
    pub author: String,
}

#[derive(Clone, Injectable)]
pub struct BookService {
    pool: Value<Pool<ConnectionManager<SqliteConnection>>>,
}

impl BookService {
    pub async fn add_book(&self, input: InsertBook) -> Result<(), AsyncError> {
        use crate::schema::books::dsl::*;
        diesel::insert_into(books)
            .values(input)
            .execute_async(&self.pool)
            .await
            .map(|_| ())
    }

    pub async fn get_book_by_id(&self, book_id: i32) -> Result<Book, AsyncError> {
        use crate::schema::books::dsl::*;
        books
            .filter(id.eq(book_id))
            .first_async::<Book>(&self.pool)
            .await
    }

    pub async fn get_books(&self) -> Result<Vec<Book>, AsyncError> {
        use crate::schema::books::dsl::*;
        books
            .load_async::<Book>(&self.pool)
            .await
    }
}
