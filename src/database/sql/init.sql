create table if not exists users (
    id bigserial primary key,
    username varchar(100) unique,
    email varchar(100) unique,
    password varchar(512),
    status integer not null,
    permissions bigint,
    created_at timestamptz default now(),
    updated_at timestamptz default now()
);