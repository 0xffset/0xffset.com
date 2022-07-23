#![feature(duration_constants)]
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

use actix_web::{
    get, middleware::Logger, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use parking_lot::Mutex;

const DELAY: Duration = Duration::from_millis(500);

enum UserState {
    Valid,
    NotFound,
}

struct State {
    pub users: Mutex<HashMap<String, Instant>>,
    pub count: Arc<AtomicUsize>,
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
        let users = state.users.lock();
        if users.contains_key(&ip) {
            let elapsed = users.get(&ip).unwrap().elapsed();

            if elapsed < DELAY {
                let count = state.count.load(Ordering::Relaxed);
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
            *state.users.lock().get_mut(&ip).unwrap() = Instant::now();
        }
        UserState::NotFound => {
            state.users.lock().insert(ip, Instant::now());
        }
    };

    let count = state.count.fetch_add(1, Ordering::Relaxed) + 1;
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
        users: Mutex::new(HashMap::new()),
        count: Arc::new(AtomicUsize::new(0)), // TODO: add count storage on disk and load it
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
