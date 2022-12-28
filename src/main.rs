use actix_web::{web::Data, App, HttpServer};
use dotenv::dotenv;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

mod services;
use services::{
    create_short_url,
    fetch_urls,
    create_short_url_table
};

pub struct AppState {
    pub db: Pool<Postgres>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create pool.");
    
    if pool.is_closed() {
        panic!("Database pool is closed.");
    } else {
        println!("Database pool is open.");
    }
    
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(AppState { db: pool.clone() }))
            .service(fetch_urls)
            .service(create_short_url)
            .service(create_short_url_table)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}