CREATE TYPE user_provider AS ENUM ('Local', 'Google');
CREATE TYPE user_role AS ENUM ('Regular', 'Premium', 'Admin');
CREATE TYPE championship_category AS ENUM ('F1', 'F2');

--- Tables
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    email VARCHAR(255) NOT NULL UNIQUE,
    username VARCHAR(20) NOT NULL,
    password VARCHAR(64),
    avatar VARCHAR(255) NOT NULL,
    provider user_provider NOT NULL DEFAULT 'Local',
    role user_role NOT NULL DEFAULT 'Regular',
    engineer BOOLEAN NOT NULL DEFAULT FALSE,
    discord_id BIGINT UNIQUE,
    active BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE championships (
    id INTEGER PRIMARY KEY,
    port INTEGER NOT NULL,
    name VARCHAR(50) NOT NULL UNIQUE,
    category championship_category NOT NULL DEFAULT 'F1',
    owner_id INTEGER NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE drivers (
    steam_name VARCHAR(100) PRIMARY KEY,
    discord_id BIGINT UNIQUE,
    nationality CHAR NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE races (
    id INTEGER PRIMARY KEY,
    championship_id INTEGER NOT NULL REFERENCES championships(id),
    name VARCHAR(100) NOT NULL,
    date TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE results (
    race_id INTEGER NOT NULL REFERENCES races(id),
    session_type SMALLINT NOT NULL,
    data BYTEA NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (race_id, session_type)
);

-- Link tables
CREATE TABLE user_championships (
    user_id INTEGER NOT NULL REFERENCES users(id),
    championship_id INTEGER NOT NULL REFERENCES championships(id),
    PRIMARY KEY (user_id, championship_id)
);

CREATE TABLE championship_drivers (
    championship_id INTEGER NOT NULL REFERENCES championships(id),
    driver_steam_name VARCHAR(100) NOT NULL REFERENCES drivers(steam_name),
    team_id SMALLINT,
    number CHAR NOT NULL,
    PRIMARY KEY (championship_id, driver_steam_name)
);

CREATE TABLE engineer_assignments (
    user_id INTEGER NOT NULL REFERENCES users(id),
    championship_id INTEGER NOT NULL REFERENCES championships(id),
    team_id SMALLINT NOT NULL,
    PRIMARY KEY (user_id, championship_id)
);

-- Indexes
CREATE INDEX ON users (email);
CREATE INDEX ON championships (id, name);
CREATE INDEX ON drivers (steam_name);
CREATE INDEX ON engineer_assignments(user_id);
CREATE INDEX ON engineer_assignments(championship_id, team_id);
CREATE INDEX ON championship_drivers (championship_id, driver_steam_name);
CREATE INDEX ON races (championship_id);
CREATE INDEX ON results (race_id);

CREATE OR REPLACE FUNCTION update_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_users_timestamp
BEFORE UPDATE ON users
FOR EACH ROW EXECUTE FUNCTION update_timestamp();

CREATE TRIGGER update_championship_timestamp
BEFORE UPDATE ON championships
FOR EACH ROW EXECUTE FUNCTION update_timestamp();

CREATE TRIGGER update_drivers_timestamp
BEFORE UPDATE ON drivers
FOR EACH ROW EXECUTE FUNCTION update_timestamp();