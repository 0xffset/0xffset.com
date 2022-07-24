#![feature(duration_constants)]
use std::io;

use actix_files::NamedFile;
use actix_web::{get, middleware::Logger, App, HttpServer};

#[get("/")]
async fn index() -> io::Result<NamedFile> {
    NamedFile::open_async("./public/html/index.html").await
}
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    let port = std::env::var("PORT")
        .expect("Missing PORT in .env")
        .parse::<u16>()
        .expect("Invalid PORT");

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn"))
        .format_timestamp(None)
        .init();

    HttpServer::new(move || {
        App::new()
            .service(index)
            .service(actix_files::Files::new("css/", "public/css"))
            .service(actix_files::Files::new("js/", "public/js"))
            .wrap(Logger::new("At `%t` from `%{r}a` to `%r`"))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
