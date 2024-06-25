use actix_cors::Cors;
use actix_files as fs;
use actix_session::{Session, SessionMiddleware};
use actix_session::storage::CookieSessionStore;
use actix_web::cookie::Key;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Error, middleware::Logger};
use actix_web_actors::ws;
use std::time::Instant;
use log::info;
use tokio::time::Duration;
use actix_web::http::header;
use actix::Actor; // Importing the Actor trait from actix

struct WsSession {
    id: usize,
    hb: Instant,
    user_id: Option<String>,
    session_id: Option<String>,
}

impl Actor for WsSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
        info!("WebSocket session started");
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        info!("WebSocket session ended");
    }
}

impl WsSession {
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(Duration::from_secs(5), |act, ctx| {
            if Instant::now().duration_since(act.hb) > Duration::from_secs(10) {
                ctx.stop();
                return;
            }
            ctx.ping(b"");
        });
    }
}

async fn ws_index(
    r: HttpRequest,
    stream: web::Payload,
    session: Session,
) -> Result<HttpResponse, Error> {
    ws::start(
        WsSession {
            id: 0,
            hb: Instant::now(),
            user_id: session.get("user_id").unwrap_or(None),
            session_id: session.get("session_id").unwrap_or(None),
        },
        &r,
        stream,
    )
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    let secret_key = Key::generate();

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT]) // Referencing header directly
            .allowed_header(header::CONTENT_TYPE)
            .max_age(3600);

        App::new()
            .wrap(Logger::default())
            .wrap(cors)
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                secret_key.clone(),
            ))
            .service(web::resource("/ws/").route(web::get().to(ws_index)))
            .service(fs::Files::new("/static", ".").show_files_listing())
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

