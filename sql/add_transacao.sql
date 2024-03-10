INSERT INTO backend.transacoes(cliente_id, valor, tipo, descricao, realizada_em)
VALUES ($1, $2, $3, $4, $5)
RETURNING $table_fields;