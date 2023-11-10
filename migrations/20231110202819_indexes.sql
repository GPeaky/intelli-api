-- Add migration script here
ALTER TABLE `user` ADD INDEX `id` (`id`);

ALTER TABLE `championship` ADD INDEX `id` (`id`);