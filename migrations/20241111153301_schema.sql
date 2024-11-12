CREATE TABLE user_table (
    id          INT AUTO_INCREMENT NOT NULL,
    user_id     VARCHAR(40) NOT NULL,
    user_name   VARCHAR(255) NOT NULL,
    mail        VARCHAR(255) NOT NULL,
    password    TEXT NOT NULL,
    PRIMARY KEY (id),
    UNIQUE KEY user_id_idx (user_id)
);

CREATE TABLE food_table (
    id          INT AUTO_INCREMENT NOT NULL,
    food_id     VARCHAR(40) NOT NULL,
    food_name   TEXT NOT NULL,
    exp         DATE NOT NULL,
    user_id     VARCHAR(40) NOT NULL,
    INDEX usr_id (user_id),
    FOREIGN KEY (user_id) REFERENCES user_table(user_id)
        ON DELETE CASCADE
        ON UPDATE CASCADE,
    PRIMARY KEY (id)
);
