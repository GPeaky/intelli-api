-- Add migration script here
ALTER TABLE `user`
    CHANGE COLUMN `avatar` `avatar` VARCHAR(100) NOT NULL COLLATE 'utf8mb4_unicode_ci' AFTER `password`;
