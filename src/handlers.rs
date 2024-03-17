use actix_web::{get, post, web, Error, HttpResponse, Responder};
use sqlx::{Acquire, PgPool};

use crate::errors::MyError;
use crate::models::*;
use crate::{db, RedBalance};

#[get("/")]
pub async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/clientes/{id}/transacoes")]
pub async fn add_transacao(
    cliente_id: web::Path<i32>,
    transacao: web::Json<Transacao>,
    db_pool: web::Data<PgPool>,
    cache_cliente: web::Data<RedBalance>,
) -> Result<HttpResponse, Error> {
    let transacao_info: Transacao = transacao.into_inner();
    transacao_info.validate_fields()?;

    let mut cliente_info = get_cliente_cache(
        cliente_id.into_inner(), 
        cache_cliente, 
        //&db_client
    ).await?;

    // let (
    //     cliente_result, 
    //     db_client_result
    // ) = tokio::join!(
    //     get_cliente_cache(
    //         cliente_id, 
    //         cache_cliente, 
    //         &db_client
    //     ),
    //     db_pool.get()
    // );
    // let mut cliente_info: Cliente = cliente_result?;
    // let db_client: Client = db_client_result.map_err(MyError::PoolError)?;

    let db_client = db_pool.into_inner();
    cliente_info
        .make_transaction(&transacao_info, &db_client)
        .await?;

    Ok(HttpResponse::Ok().json(cliente_info))
}

#[get("/clientes/{id}/extrato")]
pub async fn get_extrato(
    cliente_id: web::Path<i32>,
    db_pool: web::Data<PgPool>,
    cache_cliente: web::Data<RedBalance>,
) -> Result<HttpResponse, Error> {

    let cliente_info: Cliente = get_cliente_cache(
        cliente_id.into_inner(), 
        cache_cliente, 
        //&db_client
    ).await?;
    // let (
    //     cliente_result, 
    //     db_client_result
    // ) = tokio::join!(
    //     get_cliente_cache(
    //         cliente_id, 
    //         cache_cliente, 
    //         &db_client
    //     ),
    //     db_pool.get()
    // );
    // let cliente_info: Cliente = cliente_result?;
    // let db_client: Client = db_client_result.map_err(MyError::PoolError)?;
    
    let db_client = db_pool.into_inner();
    let mut _db_conn = db_client.acquire().await.map_err(MyError::PoolError)?;
    let db_conn = _db_conn.acquire().await.map_err(MyError::PoolError)?;
    let transacoes_info: Vec<Transacao> = db::get_transacoes(db_conn, cliente_info.id).await?;

    let extrato_info: Extrato = Extrato::build_history(cliente_info.limite, transacoes_info);

    Ok(HttpResponse::Ok().json(extrato_info))
}

async fn get_cliente_cache(
    cliente_id: i32,
    cache_cliente: web::Data<RedBalance>,
    //db_client: &PgPool,
) -> Result<Cliente, Error> {
    // check if user and his limit is cached
    // let cache_read_lock = cache_cliente.read().unwrap();
    // let cliente_cached = cache_read_lock.get(&cliente_id);
    let cliente_cached = cache_cliente.get(&cliente_id);
    let cliente_info: Cliente = match cliente_cached {
        Some(limite) => {
            //println!("Cliente found in cache");
            Cliente {
                id: cliente_id,
                limite: *limite,
                saldo: 0 
            }
        }
        None => {
            println!("Cliente {cliente_id} not found in cache");
            Err(MyError::NotFound)
            // drop(cache_read_lock);
            // let cliente = db::get_cliente(db_client, cliente_id).await?;
            // let mut cache_write_lock = cache_cliente.write().unwrap();
            // cache_write_lock.insert(cliente.id, cliente.limite);
            // cliente
        }?
    };

    Ok(cliente_info)
}