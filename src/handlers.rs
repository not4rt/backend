use actix_web::{get, post, web, Error, HttpResponse, Responder};
use deadpool_postgres::{Client, Pool};


use crate::{db, RedBalance};
use crate::errors::MyError;
use crate::models::*;

#[get("/")]
pub async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/clientes/{id}/transacoes")]
pub async fn add_transacao(
    cliente_id: web::Path<i32>,
    transacao: web::Json<Transacao>,
    db_pool: web::Data<Pool>,
    cache_cliente_debit: web::Data<RedBalance>
) -> Result<HttpResponse, Error> {
    let transacao_info: Transacao = transacao.into_inner();
    transacao_info.validate_transaction()?;

    let cache_read_lock = cache_cliente_debit.read().unwrap();
    let cliente_cache = cache_read_lock.get(&cliente_id).clone();
    if transacao_info.tipo == "d" && cliente_cache.is_some_and(|limit_exceeded| limit_exceeded == &true) {
        //println!("Cached 422");
        return Err(MyError::Unprocessable.into());
    }

    let cache_empty = cliente_cache.is_none();
    drop(cache_read_lock);

    let db_client: Client = db_pool.get().await.map_err(MyError::PoolError)?;


    let mut cliente_info: Cliente = Cliente { 
        id: cliente_id.into_inner(), 
        ..Default::default() 
    };
    let mut limit_reached = false;


    if cache_empty {
        cliente_info = db::get_cliente(&db_client, cliente_info.id).await?;
        let mut cache_write_lock = cache_cliente_debit.write().unwrap();
        limit_reached = cliente_info.saldo <= -cliente_info.limite;
        cache_write_lock.insert(cliente_info.id, limit_reached);
    }
    
    cliente_info.make_transaction(&transacao_info, &db_client).await?;
    if limit_reached == false && cliente_info.saldo <= -cliente_info.limite {
        let mut cache_write_lock = cache_cliente_debit.write().unwrap();
        limit_reached = true;
        cache_write_lock.insert(cliente_info.id, limit_reached);
    }

    Ok(HttpResponse::Ok().json(cliente_info))
}

#[get("/clientes/{id}/extrato")]
pub async fn get_extrato(
    cliente_id: web::Path<i32>,
    db_pool: web::Data<Pool>,
) -> Result<HttpResponse, Error> {
    let db_client: Client = db_pool.get().await.map_err(MyError::PoolError)?;

    let cliente_info: Cliente = db::get_cliente(&db_client, cliente_id.into_inner()).await?;

    let transacoes_info: Vec<Transacao> = db::get_transacoes(&db_client, cliente_info.id).await?;

    let extrato_info: Extrato = Extrato::build_history(cliente_info, transacoes_info);

    Ok(HttpResponse::Ok().json(extrato_info))
}