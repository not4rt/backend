use serde::{Deserialize, Serialize};
use chrono::Utc;
use sqlx::{Acquire, PgConnection, PgPool};

use crate::{db, errors::MyError};

#[derive(Clone, Copy, Deserialize, Serialize, Default)]
#[derive(sqlx::FromRow)]
pub struct Cliente {
    #[serde(skip_serializing, skip_deserializing, default)]
    pub id: i32,
    #[serde(default)]
    pub limite: i32,
    #[serde(default)]
    pub saldo: i32
}

impl Cliente {
    pub async fn get_saldo(&mut self, db_conn: &mut PgConnection) -> Result<(), MyError> {
        self.saldo = db::get_cliente_saldo(
            db_conn,
            self.id)
            .await?;
        
        Ok(())
    }
    pub async fn make_transaction(&mut self, transacao: &Transacao, db_client: &PgPool) -> Result<(), MyError> {
        let mut _db_conn = db_client.acquire().await.map_err(MyError::PoolError)?;
        let db_conn = _db_conn.acquire().await.map_err(MyError::PoolError)?;        
        
        self.get_saldo(db_conn).await?;
        if transacao.tipo == "d" && (self.saldo - transacao.valor) < -self.limite {
            return Err(MyError::Unprocessable)
        }
        self.saldo = db::make_transaction(
            db_conn,
            self.id, 
            transacao)
            .await?;
        
        //println!("id={} saldo={} trans_tipo={} trans_valor={} limite={}", self.id, self.saldo, transacao.tipo, transacao.valor, self.limite);
        Ok(())
    }
}


#[derive(Deserialize, Serialize)]
#[derive(sqlx::FromRow)]
pub struct Transacao {
    #[serde(skip_serializing, skip_deserializing)]
    pub id: i32,
    pub valor: i32,
    pub tipo: String,
    pub descricao: String,
    #[serde(skip_deserializing)]
    pub saldo_rmsc: i32,
    #[serde(default = "get_utc", skip_deserializing)]
    pub realizada_em: String
}

impl Transacao {
    pub fn validate_fields(&self)-> Result<(), MyError> {
        // valor deve ser um número inteiro positivo que representa centavos (não vamos trabalhar com frações de centavos). Por exemplo, R$ 10 são 1000 centavos
        if self.valor < 1 {
            return Err(MyError::Unprocessable);
        };
        // tipo deve ser apenas c para crédito ou d para débito
        if self.tipo != "d" && self.tipo != "c" {
            return Err(MyError::Unprocessable);
        };
        // descricao deve ser uma string de 1 a 10 caracteres
        let desc_len = self.descricao.len();
        if !(1..=10).contains(&desc_len) {
            return Err(MyError::Unprocessable);
        }
        // Uma transação de débito nunca pode deixar o saldo do cliente menor que seu limite disponível
        // Checagem movida para o postgres
        // if self.tipo == "d" && (self.saldo - self.valor) < -self.limite {
        //     return Err(MyError::Unprocessable);
        // };

        Ok(())
    }
}

fn get_utc() -> String {
    Utc::now().format("%Y-%m-%dT%H:%M:%S%.6fZ").to_string()
}

#[derive(Deserialize, Serialize)]
pub struct Saldo {
    total: i32,
    #[serde(default = "get_utc")]
    data_extrato: String,
    limite: i32
}
#[derive(Serialize)]
pub struct Extrato {
    pub saldo: Saldo,
    pub ultimas_transacoes: Vec<Transacao,>
}

impl Extrato {
    pub fn build_history(limite: i32, transacoes_vec: Vec<Transacao,>) -> Extrato {
        Extrato {
            saldo: Saldo {
                //GAMBIARRA PARA QUANDO O CLIENTE NÃO TEM TRANSACAO
                total: if !transacoes_vec.is_empty() {transacoes_vec[0].saldo_rmsc} else {0},
                data_extrato: get_utc(),
                limite
            },
            ultimas_transacoes: transacoes_vec
        }
    }
}