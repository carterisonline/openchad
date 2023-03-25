select message, role from ChatHistory
where username = $1
order by timestamp asc
limit 2 offset (select count(*) from ChatHistory where username = $1) - 2