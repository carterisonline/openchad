select message, role from ChatHistory
where username = $1
order by timestamp asc