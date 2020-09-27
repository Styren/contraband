use contraband::{Injectable, controller};
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use crate::service::{BookService, InsertBook};
use actix_web::{web, HttpResponse};

#[derive(Clone, Injectable)]
pub struct BookController {
    book_service: Arc<BookService>
}

#[derive(Serialize, Deserialize)]
pub struct NewBookInput {
    pub title: String,
    pub author: String,
}

#[controller("book")]
impl BookController {
    #[get]
    async fn get_books(self) -> HttpResponse {
        let books = self.book_service.get_books().await;
        match books {
            Ok(ok) => HttpResponse::Ok().json(ok),
            Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
        }
    }

    #[get("/{id}")]
    async fn get_book_by_id(self, id: web::Path<i32>) -> HttpResponse {
        let books = self.book_service.get_book_by_id(*id).await;
        match books {
            Ok(ok) => HttpResponse::Ok().json(ok),
            Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
        }
    }

    #[post]
    async fn add_book(self, input: web::Json<NewBookInput>) -> HttpResponse {
        match self.book_service.add_book(InsertBook {
            title: input.title.clone(),
            author: input.author.clone(),
        }).await {
            Ok(()) => HttpResponse::Ok().finish(),
            Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
        }
    }
}
