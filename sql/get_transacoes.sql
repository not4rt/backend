SELECT id, valor, tipo, descricao, saldo_rmsc, realizada_em 
FROM backend.transacoes 
WHERE cliente_id = $1 
ORDER BY id DESC 
LIMIT 10;