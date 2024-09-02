-- Enums
CREATE TYPE user_provider AS ENUM ('Local', 'Discord');
CREATE TYPE user_role AS ENUM ('Regular', 'Premium', 'Admin');
CREATE TYPE championship_category AS ENUM ('F1', 'F2');

-- Tables
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

CREATE TABLE drivers (
    steam_name VARCHAR(100) PRIMARY KEY,
    discord_id BIGINT UNIQUE,
    nationality CHAR NOT NULL,
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
    team_id SMALLINT,
    PRIMARY KEY (user_id, championship_id)
);

CREATE TABLE championship_drivers (
    championship_id INTEGER NOT NULL REFERENCES championships(id),
    driver_steam_name VARCHAR(100) NOT NULL REFERENCES drivers(steam_name),
    team_id SMALLINT,
    number CHAR NOT NULL,
    PRIMARY KEY (championship_id, driver_steam_name)
);

-- Optimized Indexes
CREATE INDEX idx_users_email ON users (email);
CREATE INDEX idx_championships_name ON championships (name);
CREATE INDEX idx_races_championship ON races (championship_id);
CREATE INDEX idx_results_race_session ON results (race_id, session_type);

-- Modify this index to be more specific if needed
CREATE INDEX idx_championship_users_team ON championship_users (team_id)
WHERE team_id IS NOT NULL;

-- Evaluate the need for this index based on your query patterns
CREATE INDEX idx_championship_drivers_team_number ON championship_drivers (championship_id, team_id, number);

-- Extended Statistics
CREATE STATISTICS ext_stats_championship_drivers (dependencies)
ON championship_id, team_id, driver_steam_name
FROM championship_drivers;

CREATE STATISTICS ext_stats_championship_users (dependencies)
ON user_id, championship_id, team_id
FROM championship_users;

CREATE STATISTICS ext_stats_results (dependencies)
ON race_id, session_type
FROM results;

CREATE STATISTICS ext_stats_races (dependencies)
ON championship_id, date
FROM races;

-- After creating extended statistics, analyze the tables
ANALYZE championship_drivers;
ANALYZE championship_users;
ANALYZE results;
ANALYZE races;