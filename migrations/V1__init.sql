-- Enums
CREATE TYPE user_provider AS ENUM ('Local', 'Discord');
CREATE TYPE user_role AS ENUM ('User', 'Premium', 'Admin');
CREATE TYPE championship_category AS ENUM ('F1', 'F2');
CREATE TYPE championship_role AS ENUM ('Visitor', 'Engineer', 'Admin');

-- Tables
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    email VARCHAR(255) NOT NULL UNIQUE,
    username VARCHAR(20) NOT NULL,
    password VARCHAR(64),
    avatar VARCHAR(255) NOT NULL,
    provider user_provider NOT NULL DEFAULT 'Local',
    role user_role NOT NULL DEFAULT 'User',
    discord_id BIGINT UNIQUE,
    active BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ
);

CREATE TABLE drivers (
    id INTEGER PRIMARY KEY,
    steam_name VARCHAR(100) NOT NULL UNIQUE,
    nationality SMALLINT NOT NULL,
    user_id INTEGER UNIQUE REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ
);

CREATE TABLE championships (
    id INTEGER PRIMARY KEY,
    port INTEGER NOT NULL,
    name VARCHAR(50) NOT NULL UNIQUE,
    category championship_category NOT NULL DEFAULT 'F1',
    owner_id INTEGER NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ
);

CREATE TABLE races (
    id INTEGER PRIMARY KEY,
    championship_id INTEGER NOT NULL REFERENCES championships(id),
    name VARCHAR(100) NOT NULL,
    date TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ
);

CREATE TABLE results (
    race_id INTEGER NOT NULL REFERENCES races(id),
    session_type SMALLINT NOT NULL,
    data BYTEA NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (race_id, session_type)
);

-- Link tables
CREATE TABLE championship_users (
    user_id INTEGER NOT NULL REFERENCES users(id),
    championship_id INTEGER NOT NULL REFERENCES championships(id),
    role championship_role NOT NULL DEFAULT 'Visitor',
    team_id SMALLINT,
    PRIMARY KEY (user_id, championship_id, role)
);

CREATE TABLE championship_drivers (
    driver_id INTEGER NOT NULL REFERENCES drivers(id),
    championship_id INTEGER NOT NULL REFERENCES championships(id),
    team_id SMALLINT,
    number SMALLINT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ,
    PRIMARY KEY (driver_id, championship_id)
);


-- Optimized indexes
CREATE INDEX idx_users_email ON users (email);
CREATE INDEX idx_users_username ON users (username);
CREATE INDEX idx_users_discord_id ON users (discord_id) WHERE discord_id IS NOT NULL;
CREATE INDEX idx_drivers_steam_name ON drivers (steam_name);
CREATE INDEX idx_drivers_user_id ON drivers (user_id) WHERE user_id IS NOT NULL;
CREATE INDEX idx_championships_name ON championships (name);
CREATE INDEX idx_championship_users ON championship_users (championship_id, role);
CREATE INDEX idx_championship_drivers ON championship_drivers (championship_id);
CREATE INDEX idx_races_championship_date ON races (championship_id, date);
CREATE INDEX idx_results_race ON results (race_id);

-- Extended statistics
CREATE STATISTICS ext_stats_championship_users (dependencies)
ON championship_id, user_id, role
FROM championship_users;

CREATE STATISTICS ext_stats_championship_drivers (dependencies)
ON championship_id, driver_id, team_id
FROM championship_drivers;

CREATE STATISTICS ext_stats_races (dependencies)
ON championship_id, date
FROM races;

ANALYZE users;
ANALYZE drivers;
ANALYZE championships;
ANALYZE championship_users;
ANALYZE championship_drivers;
ANALYZE races;
ANALYZE results;
