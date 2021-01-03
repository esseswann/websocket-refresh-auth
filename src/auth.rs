use std::collections::HashMap;
use std::collections::hash_map::Entry;
use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, decode, Header, Algorithm, errors, TokenData, Validation, EncodingKey, DecodingKey};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// pub type UserKey = (String, String);
pub type Users = HashMap<String, String>;
pub struct Auth {
    pub users: Users,
    claims: Option<Claims>
}

impl Auth {
    pub fn new(users: Users) -> Auth {
        Auth {
            users: users,
            claims: None
        }
    }
}

pub trait MessageHandler {
    fn handle_message(&mut self, string: String) -> String;
    // fn get_by_token(&mut self, string: String) -> Result<String, String>;
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
enum Message {
    Login {
        username: String,
        password: String
    },
    Logout,
    RefreshToken {
        token: String
    }
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
enum Response {
    InvalidRequest,
    InvalidToken,
    Success { token: String },
    Registered { token: String },
    InvalidPassword,
    LoggedOut,
    NotLoggedIn,
    AlreadyAuthorized { username: String }
}

impl MessageHandler for Auth {
    fn handle_message(&mut self, string: String) -> String {
        let result = match serde_json::from_str(&string) {
            Err(_) => Response::InvalidRequest,
            Ok(message) => match message {
                Message::RefreshToken { token } => {
                    let decoded: Result<TokenData<Claims>, errors::Error> = decode(
                        &token,
                        &DecodingKey::from_secret("secret".as_ref()),
                        &Validation::default());
                    match decoded {
                        Ok(token_data) => {
                            let TokenResult { token, claims } = generate_jwt(token_data.claims.sub);
                            self.claims = Some(claims);
                            Response::Success { token }
                        },
                        Err(_) => Response::InvalidToken
                    }
                },
                Message::Logout if self.claims.is_some() => {
                    self.claims = None;
                    Response::LoggedOut
                }
                Message::Logout => Response::NotLoggedIn,
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
                            match entry {
                                Entry::Occupied(_) => Response::Success { token },
                                Entry::Vacant(entry) => {
                                    entry.insert(password);
                                    Response::Registered { token }
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
    exp: u128
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

    let token = encode(&Header::default(), &my_claims, &EncodingKey::from_secret("secret".as_ref())).unwrap();

    TokenResult {
        token: token,
        claims: my_claims
    }
}

fn generate_exp() -> u128 {
    SystemTime::now()
        .checked_add(Duration::new(3600, 0)).unwrap()
        .duration_since(UNIX_EPOCH).unwrap()
        .as_millis()
}