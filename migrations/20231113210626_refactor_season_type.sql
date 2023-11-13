-- Add migration script here

ALTER TABLE championship
ALTER COLUMN
    season TYPE SMALLINT USING season:: SMALLINT;