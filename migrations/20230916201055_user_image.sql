-- Add migration script here
ALTER TABLE `user`
    ADD COLUMN `avatar` VARCHAR(100) NOT NULL AFTER `role`;
