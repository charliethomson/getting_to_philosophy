use soup::{NodeExt, QueryBuilderExt, Soup};
use std::{collections::HashMap, time::Instant};
use urlencoding::decode;
use wikipedia::{http::default::Client, Wikipedia};
const MAX_REPEATS: u8 = 4;

#[macro_use]
extern crate diesel;

mod app;
mod db;
mod path;
mod routes;

// fn get_next_article_name(wiki: &Wikipedia<Client>, article_name: String) -> String {
//     // Get the article content
//     let start = Instant::now();
//     let content = wiki
//         .page_from_title(article_name)
//         .get_html_content()
//         .expect("Failed to get article content");
//     println!("Getting content took: {}ms", start.elapsed().as_millis());

//     let start = Instant::now();
//     // Get all the p tags in the content section of the article
//     let content_p_tags = Soup::new(&content)
//         // Get the content div
//         .tag("div")
//         .class("mw-parser-output")
//         .find()
//         .unwrap()
//         // Go over each child, if it's a p, keep it, then convert the Node into the html representation
//         .children()
//         .filter(|child| child.name() == "p")
//         .map(|child| child.display())
//         .collect::<Vec<_>>()
//         .join("");

//     // Wrap them in a div, create a new soup instance for it.
//     let content_element_str = format!("<div>{}</div>", content_p_tags);
//     let content_element = Soup::new(&content_element_str);

//     // Get the first 50 a tags in the content section
//     let a_tags = content_element.tag("a").limit(50).find_all();
//     println!("Getting a tags took: {}ms", start.elapsed().as_millis());

//     let start = Instant::now();
//     // Remove all tags with classes applied to them
//     let classless_tags = a_tags.filter(|node| node.attrs().get("class").is_none());

//     // Remove all tags where the href contains a ':' (/wiki/Wikipedia:Substitution)
//     let viable_tags = classless_tags.filter_map(|node| {
//         node.attrs()
//             .get("href")
//             .map(|href_value| {
//                 // Remove tags that:
//                 //  Contain a ":" or "#"
//                 //  Do not contain "/wiki/"
//                 if [":", "#"]
//                     .into_iter()
//                     .any(|check| href_value.contains(check))
//                 {
//                     None
//                 } else if href_value.contains("/wiki/") {
//                     Some(href_value)
//                 } else {
//                     None
//                 }
//             })
//             .unwrap()
//             .cloned()
//     });

//     // Cut out the /wiki/ prefix
//     let mut article_names =
//         viable_tags.map(|link| link.split("/wiki/").nth(1).unwrap().to_string());
//     println!(
//         "Getting article names took: {}ms",
//         start.elapsed().as_millis()
//     );

//     // UrlDecode the article name (fix unicode shit)
//     return decode(&article_names.nth(0).unwrap())
//         .expect("Failed urldecoding")
//         .into_owned();
// }

// fn main() {
//     let wiki = Wikipedia::<Client>::default();
//     let mut current_article = "Bean".to_owned();

//     let mut seen_articles = HashMap::new();

//     while current_article != "Philosophy" {
//         if seen_articles.contains_key(&current_article) {
//             println!("Encountered loop");
//             break;
//         }

//         seen_articles.insert(current_article.clone(), true);
//         println!("{}", current_article);
//         current_article = get_next_article_name(&wiki, current_article.clone());
//     }
//     println!("{}", current_article);
// }

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    app::run().await;
    Ok(())
}
