use actix_web::{get, post, put, delete, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Book {
    id: u32,
    title: String,
    author: String,
}

#[get("/books")]
async fn get_books() -> impl Responder {
    HttpResponse::Ok().json(vec![
        Book { id: 1, title: "1984".to_string(), author: "George Orwell".to_string() },
        Book { id: 2, title: "To Kill a Mockingbird".to_string(), author: "Harper Lee".to_string() },
    ])
}

#[get("/books/{id}")]
async fn get_book(path: web::Path<u32>) -> impl Responder {
    let id = path.into_inner();
    HttpResponse::Ok().json(Book { id, title: "1984".to_string(), author: "George Orwell".to_string() })
}

#[post("/books")]
async fn create_book(book: web::Json<Book>) -> impl Responder {
    HttpResponse::Created().json(book.into_inner())
}

#[put("/books/{id}")]
async fn update_book(path: web::Path<u32>, book: web::Json<Book>) -> impl Responder {
    let id = path.into_inner();
    let mut updated_book = book.into_inner();
    updated_book.id = id;
    HttpResponse::Ok().json(updated_book)
}

#[delete("/books/{id}")]
async fn delete_book(path: web::Path<u32>) -> impl Responder {
    let id = path.into_inner();
    HttpResponse::Ok().body(format!("Book with id {} deleted", id))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(get_books)
            .service(get_book)
            .service(create_book)
            .service(update_book)
            .service(delete_book)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App};

    #[actix_rt::test]
    async fn test_get_books() {
        let mut app = test::init_service(
            App::new().service(get_books)
        ).await;

        let req = test::TestRequest::get().uri("/books").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_rt::test]
    async fn test_get_book() {
        let mut app = test::init_service(
            App::new().service(get_book)
        ).await;

        let req = test::TestRequest::get().uri("/books/1").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_rt::test]
    async fn test_create_book() {
        let mut app = test::init_service(
            App::new().service(create_book)
        ).await;

        let book = Book {
            id: 3,
            title: "The Great Gatsby".to_string(),
            author: "F. Scott Fitzgerald".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/books")
            .set_json(&book)
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), 201);
    }

    #[actix_rt::test]
    async fn test_update_book() {
        let mut app = test::init_service(
            App::new().service(update_book)
        ).await;

        let book = Book {
            id: 1,
            title: "1984 (Updated)".to_string(),
            author: "George Orwell".to_string(),
        };

        let req = test::TestRequest::put()
            .uri("/books/1")
            .set_json(&book)
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_rt::test]
    async fn test_delete_book() {
        let mut app = test::init_service(
            App::new().service(delete_book)
        ).await;

        let req = test::TestRequest::delete().uri("/books/1").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success());
    }
}

