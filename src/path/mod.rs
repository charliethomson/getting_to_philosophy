use std::collections::{HashMap, HashSet};

use crate::db::models::Article;
use diesel::{insert_or_ignore_into, ExpressionMethods, MysqlConnection, QueryDsl, RunQueryDsl};
use serde::{Deserialize, Serialize};
use soup::{NodeExt, QueryBuilderExt, Soup};
use urlencoding::decode;
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Path {
    Convergent { path: Vec<Article> },
    Loop { path: Vec<Article> },
}
impl Path {
    pub fn get_path(self) -> Vec<Article> {
        match self {
            Self::Convergent { path } => path,
            Self::Loop { path } => path,
        }
    }

    pub fn is_loop(&self) -> bool {
        match self {
            Self::Loop { .. } => true,
            _ => false,
        }
    }
}

fn get_page_html(article_name: String) -> String {
    reqwest::blocking::get(format!("https://en.wikipedia.org/wiki/{}", article_name))
        .unwrap()
        .text()
        .unwrap()
}

fn get_next_article_name(article_name: String) -> String {
    // Get the article content

    let content = get_page_html(article_name);
    // let content = wiki
    //     .page_from_title(article_name)
    //     .get_html_content()
    //     .expect("Failed to get article content");

    // Get all the p tags in the content section of the article
    let content_p_tags = Soup::new(&content)
        // Get the content div
        .tag("div")
        .class("mw-parser-output")
        .find()
        .unwrap()
        // Go over each child, if it's a p, keep it, then convert the Node into the html representation
        .children()
        .filter(|child| child.name() == "p")
        .map(|child| child.display())
        .collect::<Vec<_>>()
        .join("");

    // Wrap them in a div, create a new soup instance for it.
    let content_element_str = format!("<div>{}</div>", content_p_tags);
    let content_element = Soup::new(&content_element_str);

    // Get the first 50 a tags in the content section
    let a_tags = content_element.tag("a").limit(50).find_all();

    // Remove all tags where the href contains a ':' (/wiki/Wikipedia:Substitution)
    let viable_tags = a_tags.filter_map(|node| {
        node.attrs()
            .get("href")
            .map(|href_value| {
                // Remove tags that:
                //  Contain a ":" or "#"
                //  Do not contain "/wiki/"
                if [":", "#"].iter().any(|check| href_value.contains(check)) {
                    None
                } else if href_value.contains("/wiki/") {
                    Some(href_value)
                } else {
                    None
                }
            })
            .unwrap()
            .cloned()
    });

    // Cut out the /wiki/ prefix
    let mut article_names =
        viable_tags.map(|link| link.split("/wiki/").nth(1).unwrap().to_string());

    // UrlDecode the article name (fix unicode shit)
    decode(&article_names.nth(0).unwrap())
        .expect("Failed urldecoding")
        .into_owned()
}

pub fn try_get_id_from_article_name(
    conn: &MysqlConnection,
    article_name: String,
) -> Option<String> {
    try_get_article_from_article_name(conn, article_name).map(|article| article.id)
}

pub fn try_get_article_from_article_name(
    conn: &MysqlConnection,
    article_name: String,
) -> Option<Article> {
    use crate::db::schema::articles;

    articles::table
        .filter(articles::article_name.eq(article_name))
        .limit(1)
        .load::<Article>(conn)
        .expect("SQL Error")
        .get(0)
        .map(|article| article.clone())
}

pub fn generate_path(conn: &MysqlConnection, article_name: &String) -> Path {
    let mut steps: Vec<Article> = Vec::new();
    let mut seen_article_names: HashMap<String, String> = HashMap::new();
    let mut current_article_name = article_name.clone();
    let mut is_loop = false;
    let mut completed_early = false;
    let mut path: Path;
    let mut database_path = None;

    while &current_article_name != "Philosophy" {
        let mut article = Article::new(conn, current_article_name.clone());
        if try_get_id_from_article_name(conn, current_article_name.clone()).is_some() {
            completed_early = true;
            let dbp = get_path(conn, &article);
            database_path = Some(dbp.clone());
            let db_steps = dbp.get_path();
            if let Some(previous_article) = steps.last_mut() {
                previous_article.next_article = Some(article.id.clone());
            }
            current_article_name = db_steps.last().clone().unwrap().article_name.clone();
            steps.extend(db_steps.into_iter());
            break;
        } else if let Some(id) = seen_article_names.get(&current_article_name) {
            let next_article = steps
                .iter()
                .find(|article| &article.id == id)
                // Unwrap should be safe, id is inserted when we push the article to the steps
                .unwrap()
                .next_article
                .clone();

            article.set_next(next_article);
            steps.push(article);
            is_loop = true;
            break;
        } else {
            // Set the last article's next field to the current article's id
            if let Some(previous_article) = steps.last_mut() {
                previous_article.next_article = Some(article.id.clone());
            }

            seen_article_names.insert(current_article_name.clone(), article.id.clone());
            steps.push(article);
        }

        current_article_name = get_next_article_name(current_article_name.clone());
    }
    if completed_early {
        if database_path.unwrap().is_loop() {
            path = Path::Loop {
                path: steps.clone(),
            }
        } else {
            path = Path::Convergent {
                path: steps.clone(),
            }
        }
    } else if is_loop {
        path = Path::Loop {
            path: steps.clone(),
        };
    } else {
        // Path is convergent, add Philosophy article to the end
        let philosophy_article = Article::new(conn, "Philosophy".into());
        if let Some(previous_article) = steps.last_mut() {
            previous_article.next_article = Some(philosophy_article.id.clone());
        }
        steps.push(philosophy_article);
        path = Path::Convergent {
            path: steps.clone(),
        };
    }

    // Push each element into the database
    use crate::db::schema::articles::dsl::articles;
    insert_or_ignore_into(articles)
        .values(&steps)
        .execute(conn)
        .unwrap();

    path
}

pub fn get_next_article_from_db(conn: &MysqlConnection, article: &Article) -> Option<Article> {
    if article.next_article.is_none() {
        return None;
    }
    let next_article_id = article.next_article.clone().unwrap();
    use crate::db::schema::articles;

    articles::table
        .filter(articles::id.eq(next_article_id))
        .limit(1)
        .load::<Article>(conn)
        .expect("SQL Error")
        .get(0)
        .cloned()
}

pub fn get_path(conn: &MysqlConnection, article: &Article) -> Path {
    let mut steps: Vec<Article> = Vec::new();
    let mut seen_articles: HashSet<Article> = HashSet::new();
    let mut current_article = Article::new(conn, article.article_name.clone());
    let mut is_loop = false;
    let mut path: Path;

    while &current_article.article_name != "Philosophy" {
        if seen_articles.contains(&current_article) {
            let next_article = steps
                .iter()
                .find(|article| article.id == current_article.id)
                // Unwrap should be safe, id is inserted when we push the article to the steps
                .unwrap()
                .next_article
                .clone();

            current_article.set_next(next_article);
            steps.push(current_article.clone());
            is_loop = true;
            break;
        } else {
            // Set the last article's next field to the current article's id
            if let Some(previous_article) = steps.last_mut() {
                previous_article.next_article = Some(current_article.id.clone());
            }

            seen_articles.insert(current_article.clone());
            steps.push(current_article.clone());
        }

        if let Some(article) = get_next_article_from_db(conn, &current_article) {
            current_article = article;
        } else {
            let next_article_name = get_next_article_name(current_article.article_name.clone());
            let next_article = Article::new(conn, next_article_name.clone());
            if let Some(id) = try_get_id_from_article_name(conn, next_article_name) {
                if let Some(previous_article) = steps.last_mut() {
                    previous_article.next_article = Some(next_article.id.clone());
                }
                current_article = next_article
            } else {
                panic!(":(")
            }
        }
    }

    if is_loop {
        path = Path::Loop {
            path: steps.clone(),
        };
    } else {
        // Path is convergent, add Philosophy article to the end
        let philosophy_article = Article::new(conn, "Philosophy".into());
        if let Some(previous_article) = steps.last_mut() {
            previous_article.next_article = Some(philosophy_article.id.clone());
        }
        steps.push(philosophy_article);
        path = Path::Convergent {
            path: steps.clone(),
        };
    }

    // Push each element into the database
    use crate::db::schema::articles::dsl::articles;
    insert_or_ignore_into(articles)
        .values(&steps)
        .execute(conn)
        .unwrap();

    path
}
