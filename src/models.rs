use chrono::Utc;
use diesel::{insert_into, prelude::*};
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use serde::{Deserialize, Serialize};

use crate::errors::MyError;

#[derive(Clone, Copy, Deserialize, Serialize, Default, Queryable, Selectable)]
#[diesel(table_name = crate::schema::backend::clientes)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Cliente {
    #[serde(skip_serializing, skip_deserializing, default)]
    pub id: i32,
    #[serde(default)]
    pub limite: i32,
    #[serde(default)]
    pub saldo: i32,
}

impl Cliente {
    pub async fn make_transaction(
        &mut self,
        transacao: &Transacao,
        db_client: &mut bb8::PooledConnection<'_, AsyncDieselConnectionManager<AsyncPgConnection>>,
    ) -> Result<(), MyError> {
        use crate::schema::backend::clientes::dsl::*;

        // self.saldo = clientes
        //     .filter(id.eq(self.id))
        //     .select(saldo)
        //     .first(db_client)
        //     .await
        //     .expect("Error loading clientes");
        // println!(
        //     "id={} saldo={} trans_tipo={} trans_valor={} limite={}",
        //     self.id, self.saldo, transacao.tipo, transacao.valor, self.limite
        // );

        // if transacao.tipo == "d" && (self.saldo - transacao.valor) < -self.limite {
        //     return Err(MyError::Unprocessable);
        // }

        if transacao.tipo == "d" {
            // if (self.saldo - transacao.valor) < -self.limite {
            //     return Err(MyError::Unprocessable);
            // };
            let test = transacao.valor - self.limite;
            self.saldo = diesel::update(clientes.filter(
                id.eq(self.id).and(saldo.ge(test))
            ))
            .set(saldo.eq(saldo - transacao.valor))
            .returning(saldo)
            .get_result(db_client)
            .await?;
            //println!("balance={} limite={}", self.saldo, self.limite)
        } else {
            self.saldo = diesel::update(clientes.filter(id.eq(self.id)))
                .set(saldo.eq(saldo + transacao.valor))
                .returning(saldo)
                .get_result(db_client)
                .await?;
        }

        transacao.register_transaction(self, db_client).await?;
        Ok(())
    }

    pub async fn get_last_transactions(
        &self,
        db_client: &mut bb8::PooledConnection<'_, AsyncDieselConnectionManager<AsyncPgConnection>>,
    ) -> Result<Vec<Transacao>, MyError> {
        use crate::schema::backend::transacoes::dsl::*;
        let extrato: Vec<Transacao> = transacoes
            .filter(cliente_id.eq(self.id))
            .select(Transacao::as_select())
            .order(id.desc())
            .limit(10)
            .load(db_client)
            .await?;

        Ok(extrato)
    }

    pub async fn get_saldo(
        &mut self,
        db_client: &mut bb8::PooledConnection<'_, AsyncDieselConnectionManager<AsyncPgConnection>>,
    ) -> Result<i32, MyError> {
        use crate::schema::backend::clientes::dsl::*;
        self.saldo = clientes
            .filter(id.eq(self.id))
            .select(saldo)
            .first(db_client)
            .await
            .expect("Error loading clientes");
        // println!(
        //     "id={} saldo={} limite={}",
        //     self.id, self.saldo, self.limite
        // );
        Ok(self.saldo)
    }
}

#[derive(Deserialize, Serialize, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::backend::transacoes)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Transacao {
    pub valor: i32,
    pub tipo: String,
    pub descricao: String,
    #[serde(skip_deserializing)]
    pub saldo_rmsc: i32,
    #[serde(default = "get_utc", skip_deserializing)]
    pub realizada_em: String,
}

impl Transacao {
    pub fn validate_fields(&self) -> Result<(), MyError> {
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
    pub async fn register_transaction(
        &self,
        cliente_info: &Cliente,
        db_client: &mut bb8::PooledConnection<'_, AsyncDieselConnectionManager<AsyncPgConnection>>,
    ) -> Result<(), MyError> {
        use crate::schema::backend::transacoes::dsl::*;
        insert_into(transacoes)
            .values((
                cliente_id.eq(cliente_info.id),
                valor.eq(&self.valor),
                tipo.eq(&self.tipo),
                descricao.eq(&self.descricao),
                saldo_rmsc.eq(cliente_info.saldo),
                realizada_em.eq(&self.realizada_em),
            ))
            .execute(db_client)
            .await?;
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
    limite: i32,
}
#[derive(Serialize)]
pub struct Extrato {
    pub saldo: Saldo,
    pub ultimas_transacoes: Vec<Transacao>,
}

impl Extrato {
    pub async fn build_history(mut cliente_info: Cliente, transacoes_vec: Vec<Transacao>, db_client: &mut bb8::PooledConnection<'_, AsyncDieselConnectionManager<AsyncPgConnection>>,) -> Extrato {
        Extrato {
            saldo: Saldo {
                //GAMBIARRA PARA QUANDO O CLIENTE NÃO TEM TRANSACAO
                total: cliente_info.get_saldo(db_client).await.unwrap(),
                data_extrato: get_utc(),
                limite: cliente_info.limite,
            },
            ultimas_transacoes: transacoes_vec,
        }
    }
}
