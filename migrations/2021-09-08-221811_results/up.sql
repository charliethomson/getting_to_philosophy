CREATE TABLE IF NOT EXISTS articles (
    id VARCHAR(36) NOT NULL,
    CONSTRAINT `id_unique_results` UNIQUE (id),
    CONSTRAINT `id_primary_key_results` PRIMARY KEY (id),
    article_name LONGTEXT NOT NULL,
    next_article VARCHAR(36) DEFAULT NULL
)