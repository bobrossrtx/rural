use actix_web::{
    get, post,
    web::{Data, Json},
    Responder, HttpResponse
};

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
    pub short_url: String,
    pub url: String,
}

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

#[post("/create-table")]
pub async fn create_short_url_table(state: Data<AppState>) -> impl Responder {
    match sqlx::query("CREATE TABLE IF NOT EXISTS urls (id SERIAL PRIMARY KEY, short_url VARCHAR(255) NOT NULL, url VARCHAR(255) NOT NULL)")
        .execute(&state.db)
        .await
    {
        Ok(_) => HttpResponse::Ok().json("Table created"),
        Err(_) => HttpResponse::InternalServerError().json("Failed to create table"),
    }
}

#[post("/urls")]
pub async fn create_short_url(state: Data<AppState>, body: Json<CreateShortUrl>) -> impl Responder {
    // let short_url = path.into_inner();

    match sqlx::query_as::<_, Url>("INSERT INTO urls (short_url, url) VALUES ($1, $2) RETURNING *")
        .bind(body.short_url.to_string())
        .bind(body.url.to_string())
        .fetch_one(&state.db)
        .await
    {
        Ok(url) => HttpResponse::Ok().json(url),
        Err(_) => HttpResponse::InternalServerError().json("Failed to create short url"),
    }
}
