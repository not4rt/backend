// @generated automatically by Diesel CLI.

pub mod backend {
    diesel::table! {
        backend.clientes (id) {
            id -> Int4,
            #[max_length = 200]
            nome -> Varchar,
            limite -> Int4,
            saldo -> Int4,
        }
    }

    diesel::table! {
        backend.transacoes (id) {
            id -> Int4,
            cliente_id -> Int4,
            valor -> Int4,
            #[max_length = 1]
            tipo -> Varchar,
            #[max_length = 10]
            descricao -> Varchar,
            saldo_rmsc -> Int4,
            #[max_length = 200]
            realizada_em -> Varchar,
        }
    }

    diesel::joinable!(transacoes -> clientes (cliente_id));

    diesel::allow_tables_to_appear_in_same_query!(
        clientes,
        transacoes,
    );
}
