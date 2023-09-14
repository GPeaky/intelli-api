-- Add migration script here
ALTER TABLE `championship`
    DROP COLUMN IF EXISTS `user_id`,
    DROP FOREIGN KEY IF EXISTS `FK__user`;

CREATE TABLE user_championships (
    user_id INT NOT NULL,
    championship_id INT NOT NULL,
    PRIMARY KEY (user_id, championship_id),
    CONSTRAINT fk_user
        FOREIGN KEY (user_id) REFERENCES user(id),
    CONSTRAINT fk_championship
        FOREIGN KEY (championship_id) REFERENCES championship(id)
);
