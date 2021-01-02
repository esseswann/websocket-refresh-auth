use actix::{Actor, StreamHandler};
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
mod auth;
use auth::{Auth, Users, MessageHandler};
// use std::sync::{};

impl Actor for Auth {
    type Context = ws::WebsocketContext<Self>;
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Auth {
    fn handle(
        &mut self,
        msg: Result<ws::Message, ws::ProtocolError>,
        ctx: &mut Self::Context,
    ) {
        match msg {
            Ok(ws::Message::Text(text)) => ctx.text(&self.handle_message(text)),
            _ => (),
        }
    }
}

async fn index(
    users: web::Data<Users>,
    last_id: web::Data<u32>,
    req: HttpRequest,
    stream: web::Payload)
-> Result<HttpResponse, Error> {
    let resp = ws::start(Auth {
        users: users.get_ref().clone(),
        authorized: false,
        last_id: last_id.get_ref().clone(),
    }, &req, stream);
    log::debug!("{:?}", resp);
    resp
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let users = Users::new();
    let last_id: u32 = 0;
    HttpServer::new(move || App::new()
            .data(users.clone())
            .data(last_id.clone())
            .route("/", web::get().to(index)))
        .bind("127.0.0.1:9001")?
        .run()
        .await
}