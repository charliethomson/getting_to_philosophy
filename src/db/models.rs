use diesel::{MysqlConnection, Queryable};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::schema::articles;
use crate::path::try_get_article_from_article_name;

#[derive(Queryable, Insertable, Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct Article {
    pub id: String,
    pub article_name: String,
    pub next_article: Option<String>,
}
impl Article {
    pub fn new(conn: &MysqlConnection, article_name: String) -> Self {
        if let Some(article) = try_get_article_from_article_name(conn, article_name.clone()) {
            article
        } else {
            Self {
                article_name,
                id: Uuid::new_v4().to_string(),
                next_article: None,
            }
        }
    }

    pub fn set_next(&mut self, next_article: Option<String>) {
        self.next_article = next_article
    }
}
