use actix::prelude::*;
use actix_files as fs;
use actix_session::{Session, SessionMiddleware};
use actix_session::storage::CookieSessionStore;
use actix_web::cookie::Key;
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_actors::ws;
use log::{info, error};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration, Instant};
use uuid::Uuid;
use rand::{Rng, rngs::ThreadRng};
use actix::fut::wrap_future;

#[derive(Serialize, Deserialize)]
struct ClientMessage {
    message: Option<String>,
    uploadedImageData: Option<String>,
    audio: Option<Vec<u8>>,
}

#[derive(Message)]
#[rtype(result = "()")]
struct ServerMessage {
    response: String,
}

struct WsSession {
    id: usize,
    hb: Instant,
    user_id: Option<String>,
    session_id: Option<String>,
    addr: Addr<WsServer>,
}

impl Actor for WsSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
        self.addr
            .send(Connect {
                addr: ctx.address().recipient(),
            })
            .into_actor(self)
            .then(|res, act, _ctx| {
                match res {
                    Ok(id) => act.id = id,
                    _ => act.id = 0,
                }
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        self.addr.do_send(Disconnect { id: self.id });
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

#[derive(Message)]
#[rtype(usize)]
struct Connect {
    addr: Recipient<ServerMessage>,
}

#[derive(Message)]
#[rtype(result = "()")]
struct Disconnect {
    id: usize,
}

#[derive(Message)]
#[rtype(result = "()")]
struct ClientRequest {
    id: usize,
    msg: ClientMessage,
}

pub struct WsServer {
    sessions: HashMap<usize, Recipient<ServerMessage>>,
    rng: ThreadRng,
}

impl WsServer {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            rng: rand::thread_rng(),
        }
    }
}

impl Actor for WsServer {
    type Context = Context<Self>;
}

impl Handler<Connect> for WsServer {
    type Result = usize;

    fn handle(&mut self, msg: Connect, _: &mut Self::Context) -> Self::Result {
        let id = self.rng.gen::<usize>();
        self.sessions.insert(id, msg.addr);
        id
    }
}

impl Handler<Disconnect> for WsServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Self::Context) {
        self.sessions.remove(&msg.id);
    }
}

impl Handler<ClientRequest> for WsServer {
    type Result = ();

    fn handle(&mut self, msg: ClientRequest, ctx: &mut Self::Context) {
        if let Some(addr) = self.sessions.get(&msg.id) {
            let message = msg.msg.message.unwrap_or_default();
            let addr = addr.clone();
            let fut = async move {
                let response = handle_llm_interaction(message).await;
                addr.do_send(ServerMessage { response }).unwrap();
            };

            ctx.spawn(wrap_future(async move {
                fut.await;
            }));
        }
    }
}

async fn handle_llm_interaction(input: String) -> String {
    // Placeholder for actual interaction logic
    format!("Processed input: {}", input)
}

pub async fn ws_index(
    r: HttpRequest,
    stream: web::Payload,
    srv: web::Data<Addr<WsServer>>,
    session: Session,
) -> Result<HttpResponse, Error> {
    ws::start(
        WsSession {
            id: 0,
            hb: Instant::now(),
            user_id: session.get("user_id").unwrap_or(None),
            session_id: session.get("session_id").unwrap_or(None),
            addr: srv.get_ref().clone(),
        },
        &r,
        stream,
    )
}

