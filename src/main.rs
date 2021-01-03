use actix::{Actor, StreamHandler};
use actix_web::{web, http, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
mod auth;
use auth::{Auth, Users, MessageHandler};

impl Actor for Auth {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Auth {
    fn handle(
        &mut self,
        msg: Result<ws::Message, ws::ProtocolError>,
        ctx: &mut Self::Context,
    ) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => ctx.text(&self.handle_message(text)),
            _ => (),
        }
    }
}

async fn ws(
    users: web::Data<Users>,
    req: HttpRequest,
    stream: web::Payload
) -> Result<HttpResponse, Error> {    
    let resp = ws::start(
        Auth::new(users.get_ref().clone()),
        &req,
        stream);
    log::warn!("{:?}", resp);
    resp
}

async fn index() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::build(http::StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../static/index.html")))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    let users = Users::new();
    let last_id: u32 = 0;
    HttpServer::new(move || App::new()
            .data(users.clone())
            .data(last_id.clone())
            .route("/", web::get().to(index))
            .route("/ws/", web::get().to(ws)))
        .bind("127.0.0.1:9001")?
        .run()
        .await
}