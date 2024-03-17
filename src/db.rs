use sqlx::PgConnection;
use sqlx::PgPool;

use crate::errors::*;
use crate::models::*;

pub async fn init_db(db_client: &PgPool) -> Result<(), MyError> {
    let stmt: &str = include_str!("../init.sql");
    let _ = sqlx::raw_sql(stmt)
    .execute(db_client)
    .await?;

    Ok(())
}

pub async fn get_transacoes(
    db_client: &mut PgConnection,
    cliente_id: i32,
) -> Result<Vec<Transacao>, MyError> {
    let stmt = include_str!("../sql/get_transacoes.sql");
    let results: Vec<Transacao> = sqlx::query_as(stmt)
        .bind(cliente_id)
        .fetch_all(db_client)
        .await?;

    Ok(results)
}

// pub async fn add_transacao(db_client: &PoolConnection<Postgres>, cliente_id:i32, transacao_info: &Transacao) -> Result<Transacao, MyError> {
//     let _stmt = include_str!("../sql/add_transacao.sql");
//     let _stmt = _stmt.replace("$table_fields", &Transacao::sql_table_fields());
//     let stmt = db_client.prepare_cached(&_stmt).await.unwrap();

//     db_client
//         .query(
//             &stmt,
//             &[
//                 &cliente_id,
//                 &transacao_info.valor,
//                 &transacao_info.tipo,
//                 &transacao_info.descricao,
//                 &transacao_info.realizada_em,
//             ],
//         )
//         .await?
//         .iter()
//         .map(|row| Transacao::from_row_ref(row).unwrap())
//         .collect::<Vec<Transacao>>()
//         .pop()
//         .ok_or(MyError::NotFound) // more applicable for SELECTs
// }

pub async fn make_transaction(
    db_client: &mut PgConnection,
    cliente_id: i32,
    transacao_info: &Transacao,
) -> Result<i32, MyError> {
    let mut stmt = include_str!("../sql/make_transaction_c.sql");
    if transacao_info.tipo == "d" {
        stmt = include_str!("../sql/make_transaction_d.sql");
    }
    let result: (Option<i32>,) = sqlx::query_as(stmt)
        .bind(cliente_id)
        .bind(transacao_info.valor)
        .bind(&transacao_info.descricao)
        .bind(&transacao_info.realizada_em)
        .fetch_one(db_client)
        .await
        .map_err(|_| MyError::Unprocessable)?;

    let saldo: i32 = result.0.ok_or(MyError::Unprocessable)?;

    Ok(saldo)
}

// pub async fn get_cliente(
//     db_client: &mut PgConnection,
//     cliente_id: i32,
// ) -> Result<Cliente, MyError> {
//     let stmt = include_str!("../sql/get_cliente.sql");

//     let result: Cliente = sqlx::query_as(stmt)
//         .bind(cliente_id)
//         .fetch_one(db_client)
//         .await
//         .map_err(|_| MyError::NotFound)?;

//     Ok(result)
// }

pub async fn get_all_clientes(db_client: &mut PgConnection) -> Result<Vec<Cliente>, MyError> {
    let stmt = include_str!("../sql/get_all_clientes.sql");

    let results: Vec<Cliente> = sqlx::query_as(stmt)
    .fetch_all(db_client)
    .await?;
    Ok(results)
}


pub async fn get_cliente_saldo(db_client: &mut PgConnection, cliente_id: i32) -> Result<i32, MyError> {
    let stmt: &str = include_str!("../sql/get_cliente_saldo.sql");

    let result: (i32,) = sqlx::query_as(stmt)
        .bind(cliente_id)
        .fetch_one(db_client)
        .await?;        

    Ok(result.0)
}

// pub async fn update_cliente(db_client: &mut PgConnection, cliente_id: i32, old_balance: i32, new_balance: i32) -> Result<(), MyError> {
//     let _stmt = include_str!("../sql/update_cliente.sql");
//     let stmt = db_client.prepare_cached(&_stmt).await.unwrap();

//     db_client
//         .query(
//             &stmt,
//             &[
//                 &cliente_id,
//                 &old_balance,
//                 &new_balance
//             ],
//         )
//         .await?;
//     Ok(())
// }
