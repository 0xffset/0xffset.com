#![feature(duration_constants)]
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use actix_web::{
    get, middleware::Logger, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use parking_lot::RwLock;

const DELAY: Duration = Duration::from_millis(500);

enum UserState {
    Valid,
    NotFound,
}

struct State {
    pub users: RwLock<HashMap<String, Instant>>,
    pub count: RwLock<usize>,
}

struct CachedPages {
    pub index_page: String,
}

#[get("/")]
async fn index(cache: web::Data<CachedPages>) -> impl Responder {
    HttpResponse::Ok().body(cache.index_page.clone())
}

#[post("/click")]
async fn click(req: HttpRequest, state: web::Data<State>) -> impl Responder {
    let connection_info = req.connection_info();
    let ip = match connection_info.realip_remote_addr() {
        Some(ip) => ip.to_string(),
        None => return HttpResponse::BadRequest().finish(),
    };

    let user_state = {
        let users = state.users.read();
        if users.contains_key(&ip) {
            let elapsed = users.get(&ip).unwrap().elapsed();

            if elapsed < DELAY {
                let count = state.count.read();
                return HttpResponse::Accepted()
                    .body(format!("[{count},{}]", (DELAY - elapsed).as_millis() + 50));
            } else {
                UserState::Valid
            }
        } else {
            UserState::NotFound
        }
    };

    match user_state {
        UserState::Valid => {
            *state.users.write().get_mut(&ip).unwrap() = Instant::now();
        }
        UserState::NotFound => {
            state.users.write().insert(ip, Instant::now());
        }
    };

    *state.count.write() += 1;
    let count = state.count.read();

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

    let index_page =
        std::fs::read_to_string("public/index.html").expect("Error reading index.html");

    let state = web::Data::new(State {
        users: RwLock::new(HashMap::new()),
        count: RwLock::new(0), // TODO: add count storage on disk and load it
    });

    let cached_pages = web::Data::new(CachedPages { index_page });

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .app_data(cached_pages.clone())
            .service(index)
            .service(click)
            .wrap(Logger::new("At `%t` from `%{r}a` to `%r`"))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
