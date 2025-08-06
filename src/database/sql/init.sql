create table if not exists users (
    id bigserial primary key,
    username varchar(100) unique,
    email varchar(100) unique,
    password varchar(512),
    status integer not null,
    use_tfa boolean not null default false,
    tfa_status integer not null default 1,
    tfa_secret varchar(64),
    permissions bigint,
    created_at timestamptz default now(),
    updated_at timestamptz default now()
);