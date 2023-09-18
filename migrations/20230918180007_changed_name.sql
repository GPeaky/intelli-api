-- Add migration script here
ALTER TABLE `user`
    CHANGE COLUMN `method` `provider` TINYINT(2) NOT NULL DEFAULT '0' AFTER `avatar`;
