use actix_web::{web, App, HttpServer};
use deadpool_postgres::Config;
use tokio_postgres::NoTls;
use std::{collections::HashMap, sync::Arc};
// use actix_web::middleware::Logger;
//use env_logger::Env;

mod models;
mod handlers;
mod db;
mod errors;

pub type RedBalance = Arc<HashMap<i32, i32>>;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // std::env::set_var("RUST_LOG", "info");
    // env_logger::init_from_env(Env::default().default_filter_or("info"));
    // std::env::set_var("SERVER_ADDR", "127.0.0.1:9999");
    // std::env::set_var("DB_HOST", "127.0.0.1");
    // std::env::set_var("DB_NAME", "rinha");
    // std::env::set_var("DB_USER", "rinha");
    // std::env::set_var("DB_PASS", "testing");

    let server_addr = std::env::var("SERVER_ADDR").expect("SERVER_ADDR must be set");
    

    let mut cfg = Config::new();
    cfg.host = Some(std::env::var("DB_HOST").expect("DB_HOST must be set"));
    cfg.port = Some(5432);
    cfg.dbname = Some(std::env::var("DB_NAME").expect("DB_NAME must be set"));
    cfg.user = Some(std::env::var("DB_USER").expect("DB_USER must be set"));
    cfg.password = Some(std::env::var("DB_PASS").expect("DB_PASS must be set"));
    cfg.pool = deadpool_postgres::PoolConfig::new(170).into();

    let pool = cfg.create_pool(None, NoTls).unwrap();
    // Cache for existing user and his limit is cached    
    let mut cache_hashmap = HashMap::new();
    // Reinit database just to test the connection
    let db_client = pool.get().await.expect(format!("Failed to connect to Postgres on {}:{}", cfg.host.unwrap(), cfg.port.unwrap()).as_str());
    db::init_db(&db_client).await.unwrap();
    // Populate Cache
    let clientes = db::get_all_clientes(&db_client).await.expect("Invalid database. Could not retrive 'clientes'!");
    drop(db_client);
    if clientes.len() > 0 {
        clientes
        .iter()
        .for_each(|cliente| { cache_hashmap.insert(cliente.id, cliente.limite);});
    }

    let cache_cliente: RedBalance = Arc::new(cache_hashmap);
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