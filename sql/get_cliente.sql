SELECT $table_fields 
FROM backend.clientes 
WHERE id = $1
LIMIT 1;