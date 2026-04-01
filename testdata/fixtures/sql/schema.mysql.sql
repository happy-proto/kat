CREATE TABLE `theme_snapshots` (
  `id` BIGINT UNSIGNED ZEROFILL NOT NULL AUTO_INCREMENT,
  `name` VARCHAR(255) CHARACTER SET utf8mb4 NOT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

REPLACE INTO `theme_snapshots` (`id`, `name`)
VALUES (1, 'Dracula');

INSERT HIGH_PRIORITY IGNORE INTO `theme_snapshots` (`id`, `name`)
VALUES (2, 'Kat');

SHOW CREATE TABLE `theme_snapshots`;
