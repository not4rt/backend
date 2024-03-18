CREATE UNLOGGED TABLE backend.transacoes (
	id      SERIAL PRIMARY KEY,
    cliente_id INTEGER REFERENCES backend.clientes(id),
	valor  INTEGER NOT NULL,
	tipo   VARCHAR(1) NOT NULL,
	descricao   VARCHAR(10) NOT NULL,
	saldo_rmsc INTEGER NOT NULL,
	realizada_em   VARCHAR(200) NOT NULL
)


CREATE INDEX idx_extrato ON backend.transacoes (id desc, cliente_id);
