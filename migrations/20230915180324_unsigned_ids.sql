-- Add migration script here
DROP TABLE `user_championships`;

ALTER TABLE `user`
    CHANGE COLUMN `id` `id` INT UNSIGNED NOT NULL FIRST;

ALTER TABLE `championship`
    CHANGE COLUMN `id` `id` INT UNSIGNED NOT NULL FIRST;

CREATE TABLE user_championships (
    user_id INT UNSIGNED NOT NULL,
    championship_id INT UNSIGNED NOT NULL,
    PRIMARY KEY (user_id, championship_id),
    CONSTRAINT fk_user
        FOREIGN KEY (user_id) REFERENCES user(id),
    CONSTRAINT fk_championship
        FOREIGN KEY (championship_id) REFERENCES championship(id)
);
