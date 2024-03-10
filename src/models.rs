use deadpool_postgres::Client;
use serde::{Deserialize, Serialize};
use tokio_pg_mapper_derive::PostgresMapper;
use chrono::Utc;

use crate::{db, errors::MyError};

#[derive(Clone, Copy, Deserialize, PostgresMapper, Serialize, Default)]
#[pg_mapper(table = "clientes")]
pub struct Cliente {
    #[serde(skip_serializing, skip_deserializing, default)]
    pub id: i32,
    #[serde(default)]
    pub limite: i32,
    #[serde(default)]
    pub saldo: i32
}

impl Cliente {
    pub async fn make_transaction(&mut self, transacao: &Transacao, db_client: &Client) -> Result<(), MyError> {
        match db::make_transaction(&db_client, self.id, &transacao).await {
            Ok(cliente) => { 
                self.saldo = cliente.saldo;
                self.limite = cliente.limite;
            },
            Err(error) => {
                Err(error)
            }?
        };
        
        Ok(())
    }

    pub fn get_balance(&self) -> Saldo {
        Saldo {
            total: self.saldo,
            data_extrato: get_utc(),
            limite: self.limite
        }
    }
}


#[derive(Deserialize, PostgresMapper, Serialize)]
#[pg_mapper(table = "transacoes")]
pub struct Transacao {
    #[serde(skip_serializing, skip_deserializing)]
    pub id: i32,
    pub valor: i32,
    pub tipo: String,
    pub descricao: String,
    #[serde(default = "get_utc", skip_deserializing)]
    pub realizada_em: String
}

impl Transacao {
    pub fn validate_transaction(&self)-> Result<(), MyError> {
        // valor deve ser um número inteiro positivo que representa centavos (não vamos trabalhar com frações de centavos). Por exemplo, R$ 10 são 1000 centavos
        if self.valor < 1 {
            return Err(MyError::Unprocessable);
        };
        // tipo deve ser apenas c para crédito ou d para débito
        if self.tipo != "d" && self.tipo != "c" {
            return Err(MyError::Unprocessable);
        };
        // descricao deve ser uma string de 1 a 10 caracteres
        if self.descricao.len() < 1 || self.descricao.len() > 10 {
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
    pub fn build_history(cliente_info: Cliente, transacoes_vec: Vec<Transacao,>) -> Extrato {
        Extrato {
            saldo: cliente_info.get_balance(),
            ultimas_transacoes: transacoes_vec
        }
    }
}