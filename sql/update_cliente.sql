UPDATE backend.clientes 
SET saldo = $3
WHERE id = $1
AND saldo = $2;