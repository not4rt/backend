DROP SCHEMA IF EXISTS backend CASCADE;

CREATE SCHEMA backend;

CREATE UNLOGGED TABLE backend.clientes (
	id      SERIAL PRIMARY KEY,
	nome    VARCHAR(200) NOT NULL,
	limite  INTEGER NOT NULL,
    saldo   INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_clientesaldo ON backend.clientes (id, saldo);

CREATE UNLOGGED TABLE backend.transacoes (
	id      SERIAL PRIMARY KEY,
    cliente_id INTEGER REFERENCES backend.clientes(id),
	valor  INTEGER NOT NULL,
	tipo   VARCHAR(1) NOT NULL,
	descricao   VARCHAR(10) NOT NULL,
	saldo_rmsc INTEGER NOT NULL,
	realizada_em   VARCHAR(200) NOT NULL
);

CREATE INDEX idx_extrato ON backend.transacoes (id desc, cliente_id);

INSERT INTO backend.clientes (nome, limite)
VALUES
  ('o barato sai caro', 1000 * 100),
  ('zan corp ltda', 800 * 100),
  ('les cruders', 10000 * 100),
  ('padaria joia de cocaia', 100000 * 100),
  ('kid mais', 5000 * 100);

