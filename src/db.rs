use deadpool_postgres::Client;
use deadpool_postgres::GenericClient;
use tokio_pg_mapper::FromTokioPostgresRow;


use crate::models::*;
use crate::errors::*;

pub async fn init_db(db_client: &Client) -> Result<(), MyError> {
    let stmt:&str = include_str!("../init.sql");
    let _ = db_client.batch_execute(stmt).await?;

    Ok(())
}

pub async fn get_transacoes(db_client: &Client, cliente_id: i32) -> Result<Vec<Transacao,>, MyError> {
    let stmt = include_str!("../sql/get_transacoes.sql");
    let stmt = stmt.replace("$table_fields", &Transacao::sql_table_fields());
    let stmt = db_client.prepare(&stmt).await.unwrap();

    let results = db_client
        .query(
            &stmt, 
            &[
                &cliente_id
            ])
        .await?
        .iter()
        .map(|row| Transacao::from_row_ref(row).unwrap())
        .collect::<Vec<Transacao>>();

    Ok(results)
}

// pub async fn add_transacao(db_client: &Client, cliente_id:i32, transacao_info: &Transacao) -> Result<Transacao, MyError> {
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

pub async fn make_transaction_d(db_client: &Client, cliente_id:i32, valor_limite:i32, transacao_info: &Transacao) -> Result<i32, MyError> {
    let _stmt = include_str!("../sql/make_transaction_d.sql");
    let stmt = db_client.prepare(&_stmt).await.unwrap();
    let result = db_client
    .query(
        &stmt,
        &[
            &cliente_id,
            &transacao_info.valor,
            &valor_limite,
            &transacao_info.descricao,
            &transacao_info.realizada_em,
        ],
    )
    .await?
    .pop()
    .ok_or(MyError::Unprocessable)?;
    
    let saldo: Option<i32> = result.get(0);
    if saldo.is_none() {
        return Err(MyError::Unprocessable)
    }

    return Ok(saldo.unwrap())
}

pub async fn make_transaction_c(db_client: &Client, cliente_id:i32, transacao_info: &Transacao) -> Result<i32, MyError> {
    let _stmt = include_str!("../sql/make_transaction_c.sql");
    let stmt = db_client.prepare(&_stmt).await.unwrap();
    let result = db_client
    .query(
        &stmt,
        &[
            &cliente_id,
            &transacao_info.valor,
            &transacao_info.descricao,
            &transacao_info.realizada_em,
        ],
    )
    .await?
    .pop()
    .ok_or(MyError::Unprocessable)?;
    
    let saldo: i32 = result.get(0);

    return Ok(saldo)
}

// pub async fn get_cliente(db_client: &Client, cliente_id: i32) -> Result<Cliente, MyError> {
//     let stmt = include_str!("../sql/get_cliente.sql");
//     let stmt = stmt.replace("$table_fields", &Cliente::sql_fields());
//     let stmt = db_client.prepare(&stmt).await.unwrap();

//     let result = db_client
//         .query(
//             &stmt, 
//             &[
//                 &cliente_id
//             ])
//         .await?
//         .iter()
//         .map(|row| Cliente::from_row_ref(row).unwrap())
//         .collect::<Vec<Cliente>>()
//         .pop()
//         .ok_or(MyError::NotFound);
//     result
// }

pub async fn get_all_clientes(db_client: &Client) -> Result<Vec<Cliente>, MyError> {
    let stmt = include_str!("../sql/get_all_clientes.sql");
    let stmt = stmt.replace("$table_fields", &Cliente::sql_fields());
    let stmt = db_client.prepare(&stmt).await.unwrap();

    let results = db_client
        .query(
            &stmt, 
            &[]
        )
        .await?
        .iter()
        .map(|row| Cliente::from_row_ref(row).unwrap())
        .collect::<Vec<Cliente>>();
    Ok(results)
}

// pub async fn update_cliente(db_client: &Client, cliente_id: i32, old_balance: i32, new_balance: i32) -> Result<(), MyError> {
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