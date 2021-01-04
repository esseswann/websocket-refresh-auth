use std::collections::HashMap;
use std::collections::hash_map::Entry;
use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, decode, Header, errors, TokenData, Validation, EncodingKey, DecodingKey};
use std::time::{Duration, SystemTime, UNIX_EPOCH, Instant};
use actix_web_actors::ws;
use actix::*;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const TOKEN_EXPIRATION_TIMEOUT: Duration = Duration::from_secs(20);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);
const DUMMY_SECRET: &str = "secret";

pub type Users = HashMap<String, String>;
pub struct Auth {
    pub users: Users,
    pub hb: Instant,
    claims: Option<Claims>
}

impl Auth {
    pub fn new(users: Users) -> Auth {
        Auth {
            users: users,
            claims: None,
            hb: Instant::now()
        }
    }
}

pub trait MessageHandler {
    fn handle_message(&mut self, string: String) -> String;
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
enum Message {
    Login {
        username: String,
        password: String
    },
    Logout {
        token: String
    },
    RefreshToken {
        token: String
    }
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
enum Response {
    InvalidRequest,
    InvalidToken,
    Success(AuthSuccess),
    Registered(AuthSuccess),
    InvalidPassword,
    LoggedOut,
    NotLoggedIn,
    AlreadyAuthorized { username: String }
}

#[derive(Serialize, Deserialize)]
struct AuthSuccess {
    token: String,
    expires_at: u64
}

impl MessageHandler for Auth {
    fn handle_message(&mut self, string: String) -> String {
        let result = match serde_json::from_str(&string) {
            Err(_) => Response::InvalidRequest,
            Ok(message) => match message {
                Message::RefreshToken { token } => {
                    let decoded: Result<TokenData<Claims>, errors::Error> = decode(
                        &token,
                        &DecodingKey::from_secret(DUMMY_SECRET.as_ref()),
                        &Validation::default());
                    match decoded {
                        Ok(token_data) => {
                            let TokenResult { token, claims } = generate_jwt(token_data.claims.sub);
                            self.claims = Some(claims);
                            Response::Success(AuthSuccess {
                                token,
                                expires_at: self.claims.as_ref().unwrap().exp
                            })
                        },
                        Err(_) => Response::InvalidToken
                    }
                },
                Message::Logout { .. } if self.claims.is_some() => {
                    self.claims = None;
                    Response::LoggedOut
                }
                Message::Logout { .. } => Response::NotLoggedIn,
                Message::Login { .. } if self.claims.is_some() =>
                    Response::AlreadyAuthorized {
                        username: self.claims.as_ref().unwrap().sub.to_owned()
                    },
                Message::Login { username, password} => 
                    match self.users.entry(username.to_owned()) {
                        Entry::Occupied(entry) if entry.get() == &password =>
                            Response::InvalidPassword,
                        entry => {
                            let TokenResult { token, claims } = generate_jwt(username);
                            self.claims = Some(claims);
                            let auth_success = AuthSuccess {
                                token,
                                expires_at: self.claims.as_ref().unwrap().exp
                            };
                            match entry {
                                Entry::Occupied(_) => Response::Success(auth_success),
                                Entry::Vacant(entry) => {
                                    entry.insert(password);
                                    Response::Registered(auth_success)
                                } 
                            }
                        }
                    }
                }
            };
        serde_json::to_string(&result).unwrap()
    }
}

/// JWT ///

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: u64
}

struct TokenResult {
    token: String,
    claims: Claims
}

fn generate_jwt(username: String) -> TokenResult {
    let my_claims = Claims {
        sub: username,
        exp: generate_exp()
    };

    let token = encode(&Header::default(), &my_claims, &EncodingKey::from_secret(DUMMY_SECRET.as_ref())).unwrap();

    TokenResult {
        token: token,
        claims: my_claims
    }
}

fn generate_exp() -> u64 {
    SystemTime::now()
        .checked_add(TOKEN_EXPIRATION_TIMEOUT).unwrap()
        .duration_since(UNIX_EPOCH).unwrap()
        .as_secs()
}

impl Auth {
    pub fn hearbeat(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                log::error!("Websocket Client heartbeat failed, disconnecting!");
                ctx.stop();
                return;
            };
            
            ctx.ping(b"");

            if let Some(Claims { sub, exp }) = act.claims.as_ref() {
                if generate_exp() - exp > TOKEN_EXPIRATION_TIMEOUT.as_secs() - HEARTBEAT_INTERVAL.as_secs() {
                    let TokenResult { token, claims } = generate_jwt(sub.into());
                    act.claims = Some(claims);
                    let payload = Response::Success(AuthSuccess {
                        token,
                        expires_at: act.claims.as_ref().unwrap().exp
                    });
                    ctx.text(serde_json::to_string(&payload).unwrap())
                }
            }
        });
    }
}