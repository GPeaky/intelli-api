-- Add migration script here
ALTER TABLE `user`
	CHANGE COLUMN `provider` `provider` TINYINT(2) UNSIGNED NOT NULL DEFAULT '0' AFTER `avatar`;
