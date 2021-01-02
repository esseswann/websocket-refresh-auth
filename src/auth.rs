use std::collections::HashMap;
use std::collections::hash_map::Entry;
use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// pub type UserKey = (String, String);
pub type Users = HashMap<String, String>;
pub struct Auth {
    pub users: Users,
    pub authorized: bool
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
    Logout
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
enum Response {
    Success {
        token: String,
        expires_at: u128,
        is_new: bool
    },
    InvalidRequest,
    InvalidPassword,
    LoggedOut,
    NotLoggedIn,
    AlreadyAuthorized {
        username: String
    }
}

impl MessageHandler for Auth {
    fn handle_message(&mut self, string: String) -> String {
        let result = match serde_json::from_str(&string) {
            Err(_) => Response::InvalidRequest,
            Ok(message) => match message {
                Message::Logout if self.authorized => {
                    self.authorized = false;
                    Response::LoggedOut
                }
                Message::Logout => Response::NotLoggedIn,
                Message::Login { .. } if self.authorized =>
                    Response::AlreadyAuthorized {
                        username: "geh".to_string()
                    },
                Message::Login { username, password} => 
                    match self.users.entry(username.to_string()) {
                        Entry::Vacant(entry) => {
                            entry.insert(password);
                            self.authorized = true;
                            Response::Success {
                                token: generate_jwt(username),
                                expires_at: generate_exp(),
                                is_new: true
                            }
                        }
                        Entry::Occupied(entry) =>
                            match entry.get() == &password {
                                true => {
                                    self.authorized = true;
                                    Response::Success {
                                        token: generate_jwt(username),
                                        expires_at: generate_exp(),
                                        is_new: false
                                    }
                                },
                                false => Response::InvalidPassword
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

fn generate_jwt(username: String) -> String {
    let my_claims = Claims {
        sub: username,
        exp: generate_exp()
    };

    match encode(&Header::default(), &my_claims, &EncodingKey::from_secret("secret".as_ref())) {
        Ok(token) => token,
        Err(_) => "geh".to_string()
    }
}

fn generate_exp() -> u128 {
    SystemTime::now()
        .checked_add(Duration::new(3600, 0)).unwrap()
        .duration_since(UNIX_EPOCH).unwrap()
        .as_millis()
}