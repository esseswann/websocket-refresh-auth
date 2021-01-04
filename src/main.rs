use actix::{Actor, StreamHandler, Running};
use std::sync::Mutex;
use actix_web::{web, http, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use std::time::{Instant};
mod auth;
use auth::{Auth, Users, MessageHandler};

impl Actor for Auth {
    type Context = ws::WebsocketContext<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        self.hearbeat(ctx);
    }
    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        Running::Stop
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Auth {
    fn handle(
        &mut self,
        msg: Result<ws::Message, ws::ProtocolError>,
        ctx: &mut Self::Context,
    ) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Text(text)) => ctx.text(&self.handle_message(text)),
            _ => (),
        }
    }
}

async fn ws(
    users: web::Data<Mutex<Users>>,
    req: HttpRequest,
    stream: web::Payload
) -> Result<HttpResponse, Error> {    
    let resp = ws::start(
        Auth::new(users),
        &req,
        stream);
    resp
}

async fn index() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::build(http::StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(include_str!("./index.html")))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    let users = web::Data::new(Mutex::new(Users::new()));
    HttpServer::new(move || App::new()
            .app_data(users.clone())
            .route("/", web::get().to(index))
            .route("/ws/", web::get().to(ws)))
        .bind("0.0.0.0:9001")?
        .run()
        .await
}