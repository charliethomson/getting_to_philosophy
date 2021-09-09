use actix_web::{get, web::Query, HttpResponse};
use diesel::{
    query_builder::Query as DbQuery, ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl,
};
use serde::{Deserialize, Serialize};

use crate::{
    db::models::Article,
    path::{generate_path, get_path, Path},
    routes::OkMessage,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct RouteGetBody {
    query: String,
}

#[get("/query")]
pub async fn route_get(Query(mut body): Query<RouteGetBody>) -> HttpResponse {
    use crate::db::schema::articles;

    let conn = crate::db::establish_connection();

    let maybe_response = articles::table
        .filter(articles::article_name.eq(body.query.clone()))
        .limit(1)
        .load::<Article>(&conn);

    match maybe_response {
        Err(e) => HttpResponse::Ok().json(OkMessage::err(e.to_string())),
        // Article not found
        Ok(articles) if articles.len() == 0 => {
            let path = generate_path(&conn, &body.query);

            HttpResponse::Ok().json(OkMessage::ok(path))
        }
        // Article found
        Ok(articles) => {
            let article = articles.get(0).unwrap();

            let path = get_path(&conn, &article);

            HttpResponse::Ok().json(OkMessage::ok(path))
        }
    }
}
