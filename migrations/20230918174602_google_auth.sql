-- Add migration script here
ALTER TABLE `user`
    CHANGE COLUMN `password` `password` VARCHAR(255) NULL COLLATE 'utf8mb4_unicode_ci' AFTER `username`,
    ADD COLUMN `method` TINYINT(2) NOT NULL DEFAULT '0' AFTER `avatar`;
