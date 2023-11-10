-- Add migration script here
ALTER TABLE `championship`
    ADD COLUMN `category` TINYINT UNSIGNED NOT NULL AFTER `name`;