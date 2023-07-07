CREATE KEYSPACE IF NOT EXISTS intelli_api WITH REPLICATION = {'class' : 'NetworkTopologyStrategy', 'replication_factor' : 1};

CREATE TABLE IF NOT EXISTS intelli_api.game_sessions (
    id bigint PRIMARY KEY,
    data BLOB
);

CREATE TABLE IF NOT EXISTS intelli_api.car_motion (
    session_id bigint PRIMARY KEY,
    data BLOB
);

CREATE TABLE IF NOT EXISTS intelli_api.lap_data (
    session_id bigint PRIMARY KEY,
    lap BLOB,
);

CREATE TABLE IF NOT EXISTS intelli_api.event_data (
    session_id bigint PRIMARY KEY,
    string_code text,
    event BLOB
);

CREATE TABLE IF NOT EXISTS intelli_api.participants_data (
    session_id bigint PRIMARY KEY,
    participants BLOB,
);

CREATE TABLE IF NOT EXISTS intelli_api.final_classification_data (
    session_id bigint PRIMARY KEY,
    classification BLOB,
);

CREATE TABLE IF NOT EXISTS intelli_api.users (
    id text,
    email text,
    username text,
    password text,
    updated_at timestamp,
    created_at timestamp,
    PRIMARY KEY (id, email)
);