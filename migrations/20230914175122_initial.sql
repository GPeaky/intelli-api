-- Add migration script here
CREATE TABLE IF NOT EXISTS user (
    id int NOT NULL,
    email VARCHAR(100) NOT NULL,
    username VARCHAR(50) NOT NULL,
    password VARCHAR(255) NOT NULL,
    role TINYINT(3) UNSIGNED NOT NULL DEFAULT '0',
    active TINYINT(2) UNSIGNED NOT NULL DEFAULT '0',
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    PRIMARY KEY (id),
    UNIQUE INDEX email (email(100))
);

CREATE TABLE IF NOT EXISTS championship (
    id int NOT NULL,
    port smallint UNSIGNED NOT NULL,
    name VARCHAR(50) NOT NULL,
    user_id int NOT NULL,
    PRIMARY KEY (id),
    UNIQUE INDEX name (name),
    CONSTRAINT `FK__user` FOREIGN KEY (`user_id`) REFERENCES `user` (`id`) ON UPDATE RESTRICT ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS event_data (
    id INT AUTO_INCREMENT PRIMARY KEY,
    session_id BIGINT NOT NULL,
    string_code CHAR(4) NOT NULL,
    event BLOB NOT NULL
);