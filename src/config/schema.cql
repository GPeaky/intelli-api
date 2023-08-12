CREATE KEYSPACE IF NOT EXISTS intelli_api WITH REPLICATION = {'class' : 'NetworkTopologyStrategy', 'replication_factor' : 1};

CREATE TABLE IF NOT EXISTS game_sessions (
    id bigint PRIMARY KEY,
    championship_id text,
    data BLOB
);

CREATE TABLE IF NOT EXISTS lap_data (
    session_id bigint PRIMARY KEY,
    lap BLOB,
);

CREATE TABLE IF NOT EXISTS event_data (
    session_id bigint,
    string_code text,
    events list<BLOB>,
    PRIMARY KEY (session_id, string_code)
);

CREATE TABLE IF NOT EXISTS participants_data (
    session_id bigint PRIMARY KEY,
    participants BLOB,
);

CREATE TABLE IF NOT EXISTS final_classification_data (
    session_id bigint PRIMARY KEY,
    classification BLOB,
);

CREATE TABLE IF NOT EXISTS users (
    id bigint,
    email text,
    username text,
    password text,
    active boolean,
    role smallint,
    updated_at timestamp,
    created_at timestamp,
    PRIMARY KEY (id, email)
);

CREATE TABLE IF NOT EXISTS championships (
    id bigint,
    port smallint,
    name text,
    user_id bigint,
    updated_at timestamp,
    created_at timestamp,
    PRIMARY KEY (id, user_id)
);