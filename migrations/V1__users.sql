CREATE TYPE user_provider AS ENUM ('Local', 'Google');

CREATE TYPE user_role AS ENUM ('Free', 'Premium', 'Business', 'Admin');

CREATE TABLE
    users (
        id INTEGER NOT NULL PRIMARY KEY,
        email VARCHAR(255) NOT NULL UNIQUE,
        username VARCHAR(20) NOT NULL,
        password VARCHAR(64),
        avatar VARCHAR(255) NOT NULL,
        provider user_provider NOT NULL DEFAULT 'Local',
        role user_role NOT NULL DEFAULT 'Free',
        active BOOLEAN NOT NULL DEFAULT FALSE,
        created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
    );

CREATE INDEX "users_email_idx" ON "users" ("email");
