use actix_web::{
    get, post, delete,
    web::{Data, Json, Path},
    Responder, HttpResponse
};

use actix_web_lab::web as web_lab;

use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use sqlx::{self, FromRow};
use crate::AppState;

#[derive(Serialize, FromRow)]
struct Url {
    id: i32,
    short_url: String,
    url: String,
}

#[derive(Deserialize)]
pub struct CreateShortUrl {
    pub url: String,
}

// Fetch all urls
#[get("/urls")]
pub async fn fetch_urls(state: Data<AppState>) -> impl Responder {
    match sqlx::query_as::<_, Url>("SELECT * FROM urls")
        .fetch_all(&state.db)
        .await
    {
        Ok(urls) => HttpResponse::Ok().json(urls),
        Err(_) => HttpResponse::NotFound().json("No urls found"),
    }
}

// Fetch url by id
#[get("/urls/{url_id}")]
pub async fn fetch_url_by_id(state: Data<AppState>, url_id: Path<i32>) -> impl Responder {
    match sqlx::query_as::<_, Url>("SELECT * FROM urls WHERE id = $1")
        .bind(url_id.into_inner())
        .fetch_one(&state.db)
        .await
    {
        Ok(url) => HttpResponse::Ok().json(url),
        Err(_) => HttpResponse::NotFound().json("Url not found"),
    }
}

// Redirect short url to end url
#[get("/u/{short_url}")]
pub async fn redirect_shorturl_to_url(state: Data<AppState>, short_url: Path<String>) -> impl Responder {
    let short_url_friendly = format!("/{short_url}");
    println!("short_url_friendly: {}", short_url_friendly);
    match sqlx::query_as::<_, Url>("SELECT * from urls where short_url = $1")
        .bind(short_url.to_string())
        .fetch_one(&state.db)
        .await
    {
        Ok(url) => web_lab::Redirect::new(short_url_friendly, url.url),
        Err(_) => web_lab::Redirect::new(short_url_friendly, "/urls/-1"),
    }
}

// Create table
#[post("/api/create-table")]
pub async fn create_short_url_table(state: Data<AppState>) -> impl Responder {
    match sqlx::query("CREATE TABLE IF NOT EXISTS urls (id SERIAL PRIMARY KEY, short_url VARCHAR(255) NOT NULL, url VARCHAR(255) NOT NULL)")
        .execute(&state.db)
        .await
    {
        Ok(_) => HttpResponse::Ok().json("Table created"),
        Err(_) => HttpResponse::InternalServerError().json("Failed to create table"),
    }
}

// Create short url
#[post("/api/urls")]
pub async fn create_short_url(state: Data<AppState>, body: Json<CreateShortUrl>) -> impl Responder {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    const SHORT_URL_LENGTH: usize = 8;

    // If url doesn't start with http:// or https://, add https://
    let redirect_url = if body.url.starts_with("http://") || body.url.starts_with("https://") {
        body.url.to_string()
    } else {
        format!("https://{}", body.url)
    };

    let mut rng = thread_rng();

    let short_url: String = (0..SHORT_URL_LENGTH)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();

    match sqlx::query_as::<_, Url>("INSERT INTO urls (short_url, url) VALUES ($1, $2) RETURNING *")
        .bind(short_url.to_string())
        .bind(redirect_url.to_string())
        .fetch_one(&state.db)
        .await
    {
        Ok(url) => HttpResponse::Ok().json(url),
        Err(_) => HttpResponse::InternalServerError().json("Failed to create short url"),
    }
}

// Delete url
#[delete("/api/urls/{url_id}")]
pub async fn delete_url(state: Data<AppState>, url_id: Path<i32>) -> impl Responder {
    match sqlx::query("DELETE FROM urls WHERE id = $1")
        .bind(url_id.into_inner())
        .execute(&state.db)
        .await
    {
        Ok(_) => HttpResponse::Ok().json("Url deleted"),
        Err(_) => HttpResponse::InternalServerError().json("Failed to delete url"),
    }
}

// Delete all urls
#[delete("/api/urls")]
pub async fn delete_all_urls(state: Data<AppState>) -> impl Responder {
    match sqlx::query("DELETE FROM urls")
        .execute(&state.db)
        .await
    {
        Ok(_) => HttpResponse::Ok().json("All urls deleted"),
        Err(_) => HttpResponse::InternalServerError().json("Failed to delete all urls"),
    }
}

