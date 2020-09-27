#[macro_use]
extern crate diesel;

use diesel::sqlite::SqliteConnection;
use contraband::core::ContrabandApp;
use contraband::module;
use contraband_diesel::DieselPoolModule;
use crate::controller::BookController;
use crate::service::BookService;

pub mod schema;
mod controller;
mod service;

type SqliteModule = DieselPoolModule<SqliteConnection>;

#[module]
#[import(SqliteModule)]
#[provider(BookService)]
#[controller(BookController)]
struct AppModule;

#[contraband::main]
async fn main() -> std::io::Result<()> {
    ContrabandApp::new()
        .start::<AppModule>()
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use service::Book;
    use controller::NewBookInput;
    use actix_web::test;

    #[contraband::test]
    async fn add_book() {
        let mut server = ContrabandApp::new()
            .test_server::<AppModule>()
            .await;

        let input = NewBookInput {
            title: "Bilbo Baggins".to_string(),
            author: "J.R.R. Tolkien".to_string(),
        };

        let post = test::TestRequest::post().uri("/book").set_json(&input).to_request();
        test::call_service(&mut server, post).await;

        let get = test::TestRequest::get().uri("/book").to_request();
        let get_resp: Vec<Book> = test::read_response_json(&mut server, get).await;

        assert_eq!(get_resp.len(), 1);
        assert_eq!(get_resp[0].title, input.title);
    }
}
