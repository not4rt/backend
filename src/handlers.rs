use actix_web::{get, post, web, Error, HttpResponse, Responder};
use deadpool_postgres::{Client, Pool};

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
    db_pool: web::Data<Pool>,
    cache_cliente: web::Data<RedBalance>,
) -> Result<HttpResponse, Error> {
    let transacao_info: Transacao = transacao.into_inner();
    transacao_info.validate_fields()?;

    let mut cliente_info = get_cliente_cache(
        cliente_id, 
        cache_cliente, 
        //&db_client
    ).await?;

    let db_client: Client = db_pool.get().await.map_err(MyError::PoolError)?;
    cliente_info
        .make_transaction(&transacao_info, &db_client)
        .await?;

    Ok(HttpResponse::Ok().json(cliente_info))
}

#[get("/clientes/{id}/extrato")]
pub async fn get_extrato(
    cliente_id: web::Path<i32>,
    db_pool: web::Data<Pool>,
    cache_cliente: web::Data<RedBalance>,
) -> Result<HttpResponse, Error> {
    let db_client: Client = db_pool.get().await.map_err(MyError::PoolError)?;

    let cliente_info: Cliente = get_cliente_cache(
        cliente_id, 
        cache_cliente, 
        //&db_client
    ).await?;

    let transacoes_info: Vec<Transacao> = db::get_transacoes(&db_client, cliente_info.id).await?;

    let extrato_info: Extrato = Extrato::build_history(cliente_info, transacoes_info);

    Ok(HttpResponse::Ok().json(extrato_info))
}

async fn get_cliente_cache(
    cliente_id: web::Path<i32>,
    cache_cliente: web::Data<RedBalance>,
    //db_client: &Client,
) -> Result<Cliente, Error> {
    // check if user and his limit is cached
    let cache_read_lock = cache_cliente.read().unwrap();
    let cliente_cached = cache_read_lock.get(&cliente_id);
    let cliente_info: Cliente = match cliente_cached {
        Some(limite) => {
            //println!("Cliente found in cache");
            Cliente {
                id: cliente_id.into_inner(),
                limite: limite.clone(),
                ..Default::default()
            }
        }
        None => {
            println!("Cliente not found in cache");
            Err(MyError::NotFound)
            //drop(cache_read_lock);
            //let cliente = db::get_cliente(&db_client, cliente_id.into_inner()).await?;
            //let mut cache_write_lock = cache_cliente.write().unwrap();
            //cache_write_lock.insert(cliente.id, cliente.limite);
            //cliente
        }?
    };

    Ok(cliente_info)
}
