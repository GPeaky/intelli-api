CREATE KEYSPACE IF NOT EXISTS intelli_api WITH REPLICATION = {'class' : 'NetworkTopologyStrategy', 'replication_factor' : 1};

CREATE TABLE IF NOT EXISTS game_sessions (
    id bigint PRIMARY KEY,
    championship_id text,
    data BLOB
);

CREATE TABLE IF NOT EXISTS car_motion (
    session_id bigint PRIMARY KEY,
    data BLOB
);

CREATE TABLE IF NOT EXISTS lap_data (
    session_id bigint PRIMARY KEY,
    lap BLOB,
);

CREATE TABLE IF NOT EXISTS event_data (
    session_id bigint PRIMARY KEY,
    string_code text,
    event BLOB
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
    id text,
    email text,
    username text,
    password text,
    active boolean,
    updated_at timestamp,
    created_at timestamp,
    PRIMARY KEY (id, email)
);

CREATE TABLE IF NOT EXISTS championships (
    id text PRIMARY KEY,
    port smallint,
    name text,
    user_id text,
    updated_at timestamp,
    created_at timestamp,
);