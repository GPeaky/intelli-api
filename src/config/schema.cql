CREATE KEYSPACE IF NOT EXISTS intelli_api WITH REPLICATION = {'class' : 'NetworkTopologyStrategy', 'replication_factor' : 1};

CREATE TABLE IF NOT EXISTS event_data (
    session_id bigint,
    string_code text,
    events list<BLOB>,
    PRIMARY KEY (session_id, string_code)
);

CREATE TABLE IF NOT EXISTS users (
    id int PRIMARY KEY,
    email text,
    username text,
    password text,
    active boolean,
    role smallint,
    updated_at timestamp,
    created_at timestamp,
);

CREATE MATERIALIZED VIEW IF NOT EXISTS users_by_email AS
SELECT * FROM users
WHERE email IS NOT NULL
PRIMARY KEY (email,id);

CREATE TABLE IF NOT EXISTS championships (
    id int,
    port smallint,
    name text,
    user_id int,
    updated_at timestamp,
    created_at timestamp,
    PRIMARY KEY (id, user_id)
);

CREATE MATERIALIZED VIEW IF NOT EXISTS championships_by_name AS
SELECT name, id, user_id
FROM championships
WHERE name IS NOT NULL AND user_id IS NOT NULL
PRIMARY KEY (name, id, user_id);