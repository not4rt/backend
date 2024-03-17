SELECT saldo 
FROM backend.clientes 
WHERE id = $1
LIMIT 1;