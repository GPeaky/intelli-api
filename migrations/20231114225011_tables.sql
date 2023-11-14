-- Add migration script here

CREATE INDEX ON "championship" ("id");

CREATE UNIQUE INDEX ON "championship" ("name");

CREATE INDEX ON "users" ("id");