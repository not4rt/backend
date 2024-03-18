use actix_web::{web, App, HttpServer};
use diesel::{QueryDsl, SelectableHelper};
use std::{collections::HashMap, sync::Arc};
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pooled_connection::bb8::Pool;
use diesel_async::RunQueryDsl;

use crate::models::Cliente;

// use actix_web::middleware::Logger;
// use env_logger::Env;

mod models;
mod handlers;
mod errors;
pub mod schema;

pub type RedBalance = Arc<HashMap<i32, i32>>;
pub type DbPoll = Pool<diesel_async::AsyncPgConnection>;

pub async fn get_connection_pool() -> Pool<diesel_async::AsyncPgConnection> {
    let pg_host = std::env::var("DB_HOST").expect("DB_HOST must be set");
    let pg_port = 5432;
    let pg_dbname = std::env::var("DB_NAME").expect("DB_NAME must be set");
    let pg_user = std::env::var("DB_USER").expect("DB_USER must be set");
    let pg_password = std::env::var("DB_PASS").expect("DB_PASS must be set");
    let pg_connection = format!("postgres://{}:{}@{}:{}/{}", pg_user, pg_password, pg_host, pg_port, pg_dbname);

    let mgr = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(pg_connection);
    Pool::builder()
    .max_size(170)
    .min_idle(Some(50))
    .build(mgr)
    .await
    .expect("Could not estabilish connection to the database...")
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // std::env::set_var("RUST_LOG", "debug");
    // env_logger::init_from_env(Env::default().default_filter_or("info"));
    // std::env::set_var("SERVER_ADDR", "127.0.0.1:9999");
    // std::env::set_var("DB_HOST", "127.0.0.1");
    // std::env::set_var("DB_NAME", "rinha");
    // std::env::set_var("DB_USER", "rinha");
    // std::env::set_var("DB_PASS", "testing");
    
    let server_addr: String = std::env::var("SERVER_ADDR").expect("SERVER_ADDR must be set");

    let pool = get_connection_pool().await;
    let pool_clone = pool.clone();
    let mut conn = &mut pool_clone.get().await.unwrap();

    let mut hashmap = HashMap::new();
    // Reinit database just to test the connection
    let stmt:&str = include_str!("../init.sql");
    diesel_async::SimpleAsyncConnection::batch_execute(&mut conn, stmt).await.unwrap();
    // Populate Cache
    use crate::schema::backend::clientes::dsl::*;
    let results: Vec<models::Cliente> = clientes
    .select(Cliente::as_select())
    .load(conn)
    .await
    .unwrap();
    if !results.is_empty() {
        println!("Results not empty");
        results
        .iter()
        .for_each(|cliente| { 
            hashmap.insert(cliente.id, cliente.limite);
            }
        );
    } else {
        println!("Results empty");
    }

    // Cache for existing user and his limit is cached    
    let cache_cliente: RedBalance = Arc::new(hashmap);

    let server = HttpServer::new(move || {
        App::new()
        .app_data(web::Data::new(pool.clone()))
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