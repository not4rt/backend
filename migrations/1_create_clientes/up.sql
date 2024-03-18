DROP SCHEMA IF EXISTS backend CASCADE;
CREATE SCHEMA backend;

CREATE UNLOGGED TABLE backend.clientes (
	id      SERIAL PRIMARY KEY,
	nome    VARCHAR(200) NOT NULL,
	limite  INTEGER NOT NULL,
    saldo   INTEGER NOT NULL DEFAULT 0
)


CREATE INDEX idx_clientesaldo ON backend.clientes (id, saldo);