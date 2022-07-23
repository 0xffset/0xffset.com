#![feature(duration_constants)]
use std::{
    collections::HashMap,
    sync::Mutex,
    time::{Duration, Instant},
};

use actix_files::NamedFile;
use actix_web::{
    get, middleware::Logger, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder,
    Result,
};

const DELAY: Duration = Duration::from_secs(5);

struct State {
    pub users: Mutex<HashMap<String, Instant>>,
    pub count: Mutex<usize>,
}

#[get("/")]
async fn index() -> Result<NamedFile> {
    Ok(NamedFile::open("public/index.html")?)
}

#[post("/click")]
async fn click(req: HttpRequest, state: web::Data<State>) -> impl Responder {
    let connection_info = req.connection_info();
    let ip = match connection_info.realip_remote_addr() {
        Some(ip) => ip.to_string(),
        None => return HttpResponse::BadRequest().finish(),
    };

    let mut users = state.users.lock().unwrap();
    let mut count = state.count.lock().unwrap();

    if users.contains_key(&ip) {
        let time = users.get_mut(&ip).unwrap();
        let elapsed = time.elapsed();
        if elapsed < DELAY {
            return HttpResponse::Accepted()
                .body(format!("[{count},{}]", (DELAY - elapsed).as_secs() + 1));
        } else {
            *time = Instant::now();
        }
    } else {
        users.insert(ip, Instant::now());
    }

    *count += 1;

    HttpResponse::Ok().body(format!("[{count},-1]"))
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

    log::info!("Starting server on port {port}...");

    let state = web::Data::new(State {
        users: Mutex::new(HashMap::new()),
        count: Mutex::new(0), // TODO: add count storage on disk and load it
    });

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .service(index)
            .service(click)
            .wrap(Logger::new("At `%t` from `%{r}a` to `%r`"))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
