use actix_web::{App, HttpServer};

const URL: &str = "localhost:3000";

pub async fn run() {
    std::env::set_var("RUST_LOG", "actix_web");

    use crate::routes::*;

    let server = HttpServer::new(move || App::new().service(route_get))
        .bind(URL)
        .unwrap();
    println!("Listening on http://{}", URL);

    server.run().await;
}
