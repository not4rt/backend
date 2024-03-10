SELECT $table_fields 
FROM backend.transacoes 
WHERE cliente_id = $1 
ORDER BY id DESC 
LIMIT 10;