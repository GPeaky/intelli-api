-- Add migration script here
ALTER TABLE `championship`
	ADD COLUMN `owner_id` INT(10) UNSIGNED NOT NULL AFTER `name`;
