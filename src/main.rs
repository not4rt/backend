use actix_web::{web, App, HttpServer};
use std::{collections::HashMap, sync::Arc};
use sqlx::{postgres::PgPoolOptions, Acquire};
// use actix_web::middleware::Logger;
// use env_logger::Env;

mod models;
mod handlers;
mod db;
mod errors;

pub type RedBalance = Arc<HashMap<i32, i32>>;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // std::env::set_var("RUST_LOG", "debug");
    // env_logger::init_from_env(Env::default().default_filter_or("info"));
    // std::env::set_var("SERVER_ADDR", "127.0.0.1:9999");
    // std::env::set_var("DB_HOST", "127.0.0.1");
    // std::env::set_var("DB_NAME", "rinha");
    // std::env::set_var("DB_USER", "rinha");
    // std::env::set_var("DB_PASS", "testing");

    let server_addr = std::env::var("SERVER_ADDR").expect("SERVER_ADDR must be set");

    let pg_host = std::env::var("DB_HOST").expect("DB_HOST must be set");
    let pg_port = 5432;
    let pg_dbname = std::env::var("DB_NAME").expect("DB_NAME must be set");
    let pg_user = std::env::var("DB_USER").expect("DB_USER must be set");
    let pg_password = std::env::var("DB_PASS").expect("DB_PASS must be set");
    let pg_connection = format!("postgres://{}:{}@{}:{}/{}", pg_user, pg_password, pg_host, pg_port, pg_dbname);

    let pool = PgPoolOptions::new()
    .max_connections(170)
    .connect(pg_connection.as_str())
    .await
    .unwrap();

    let mut hashmap = HashMap::new();
    // Reinit database just to test the connection
    db::init_db(&pool).await.unwrap();
    // Populate Cache
    let clientes = db::get_all_clientes(pool.acquire().await.unwrap().acquire().await.unwrap()).await.expect("Invalid database. Could not retrive 'clientes'!");
    if !clientes.is_empty() {
        clientes
        .iter()
        .for_each(|cliente| { 
            hashmap.insert(cliente.id, cliente.limite);
            }
        );
    }

    // Cache for existing user and his limit is cached    
    let cache_cliente: RedBalance = Arc::new(hashmap);

    let server = HttpServer::new(move || {
        let web_data_pool = web::Data::new(pool.clone());
        App::new()
        .app_data(web_data_pool)
        .app_data(web::Data::new(cache_cliente.clone()))
            .service(handlers::hello)
            .service(handlers::add_transacao)
            .service(handlers::get_extrato)
            // .wrap(Logger::default())
    })
    .bind(server_addr.clone())?
    .run();
    println!("Server running at http://{}/", server_addr);

    server.await
}