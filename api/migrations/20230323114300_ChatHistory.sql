create table if not exists ChatHistory (
    username text,
    message text,
    role text check (role in ('system', 'assistant', 'user')),
    timestamp datetime default current_timestamp
);