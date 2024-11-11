use actix_web::{get, post, put, delete, web, App, HttpResponse, HttpServer, Responder, http::StatusCode};
use serde::{Deserialize, Serialize};
use postgrest::Postgrest;
use dotenv::dotenv;
use std::env;
use serde_json;

// 기존 Book 구조체는 그대로 유지
#[derive(Debug, Serialize, Deserialize)]
struct Book {
    id: u32,
    title: String,
    author: String,
}

// Supabase 클라이언트를 위한 새로운 구조체
#[derive(Clone)]
struct SupabaseClient {
    client: Postgrest,
}

impl SupabaseClient {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        dotenv().ok();
        let supabase_url = env::var("SUPABASE_URL")?;
        let supabase_key = env::var("SUPABASE_KEY")?;

        let client = Postgrest::new(supabase_url)
            .insert_header("apikey", &supabase_key)
            .insert_header("Content-Type", "application/json");

        // 환경 변수 로드 시도를 로깅
        println!("Loading environment variables...");
        
        let supabase_url = match env::var("SUPABASE_URL") {
            Ok(url) => {
                println!("✅ SUPABASE_URL loaded successfully");
                let url = if !url.ends_with("/rest/v1") {
                    format!("{}/rest/v1", url)
                } else {
                    url
                };
                url
            },
            Err(e) => {
                println!("❌ Failed to load SUPABASE_URL: {:?}", e);
                return Err(Box::new(e));
            }
        };

        let supabase_key = match env::var("SUPABASE_KEY") {
            Ok(key) => {
                println!("✅ SUPABASE_KEY loaded successfully");
                key
            },
            Err(e) => {
                println!("❌ Failed to load SUPABASE_KEY: {:?}", e);
                return Err(Box::new(e));
            }
        };

        println!("🔄 Initializing Supabase client...");
        let client = Postgrest::new(supabase_url)
            .insert_header("apikey", &supabase_key)
            .insert_header("Authorization", format!("Bearer {}", supabase_key))
            .insert_header("Content-Type", "application/json");
        
        println!("✅ Supabase client initialized successfully");

        Ok(SupabaseClient { client })
    }

    // Supabase 관련 메서드를 여기에 추가할 수 있습니다.
}

// 기존의 API 핸들러들은 그대로 유지
// get_books, get_book, create_book, update_book, delete_book

#[post("/books")]
async fn create_book(supabase: web::Data<SupabaseClient>, book: web::Json<Book>) -> impl Responder {
    let new_book = book.into_inner();
    
    let json_data = serde_json::json!({
        "title": new_book.title,
        "author": new_book.author,
    });

    let json_string = serde_json::to_string(&json_data).unwrap();

    println!("📝 Attempting to insert book: {}", json_string);

    match supabase.client
        .from("books")
        .insert(json_string)
        .execute()
        .await
    {
        Ok(response) => {
            println!("📨 Response received: {:?}", response);
            println!("📊 Response status: {:?}", response.status());
            println!("🔍 Response headers: {:?}", response.headers());

            // 상태 코드를 먼저 확인
            let status = response.status();
            
            // 응답 본문 로깅
            if let Ok(text) = response.text().await {
                println!("📄 Response body: {}", text);
            }

            match status {
                StatusCode::CREATED => HttpResponse::Created().json(new_book),
                _ => {
                    println!("⚠️ Unexpected status code: {:?}", status);
                    HttpResponse::InternalServerError().finish()
                }
            }
        },
        Err(e) => {
            println!("❌ Error during request: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Supabase 클라이언트 초기화
    let supabase_client = match SupabaseClient::new() {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Failed to initialize Supabase client: {}", e);
            return Ok(());
        }
    };

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(supabase_client.clone()))
            .service(create_book)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

// 기존의 테스트 코드는 그대로 유지
#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App};

    // #[actix_rt::test]
    // async fn test_get_books() {
    //     let mut app = test::init_service(
    //         App::new().service(get_books)
    //     ).await;

    //     let req = test::TestRequest::get().uri("/books").to_request();
    //     let resp = test::call_service(&mut app, req).await;
    //     assert!(resp.status().is_success());
    // }

    // #[actix_rt::test]
    // async fn test_get_book() {
    //     let mut app = test::init_service(
    //         App::new().service(get_book)
    //     ).await;

    //     let req = test::TestRequest::get().uri("/books/1").to_request();
    //     let resp = test::call_service(&mut app, req).await;
    //     assert!(resp.status().is_success());
    // }

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

    // #[actix_rt::test]
    // async fn test_update_book() {
    //     let mut app = test::init_service(
    //         App::new().service(update_book)
    //     ).await;

    //     let book = Book {
    //         id: 1,
    //         title: "1984 (Updated)".to_string(),
    //         author: "George Orwell".to_string(),
    //     };

    //     let req = test::TestRequest::put()
    //         .uri("/books/1")
    //         .set_json(&book)
    //         .to_request();
    //     let resp = test::call_service(&mut app, req).await;
    //     assert!(resp.status().is_success());
    // }

    // #[actix_rt::test]
    // async fn test_delete_book() {
    //     let mut app = test::init_service(
    //         App::new().service(delete_book)
    //     ).await;

    //     let req = test::TestRequest::delete().uri("/books/1").to_request();
    //     let resp = test::call_service(&mut app, req).await;
    //     assert!(resp.status().is_success());
    // }
}

