-- Add migration script here
ALTER TABLE `championship`
    ADD COLUMN `season` TINYINT UNSIGNED NOT NULL AFTER `name`,
    ADD COLUMN `driver_count` SMALLINT UNSIGNED NOT NULL AFTER `season`;
