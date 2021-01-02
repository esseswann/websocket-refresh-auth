use std::collections::HashMap;
use std::collections::hash_map::Entry;
use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub type UserKey = (String, String);
pub type Users = HashMap<UserKey, u32>;
pub struct Auth {
    pub users: Users,
    pub authorized: bool,
    pub last_id: u32,
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
    Fail
}

impl MessageHandler for Auth {
    fn handle_message(&mut self, string: String) -> String {
        match serde_json::from_str(&string) {
            Err(_) => "Invalid request".to_string(),
            Ok(message) => match message {
                Message::Login { username, password} => 
                    match self.users.entry((username, password)) {
                        Entry::Vacant(entry) => {
                            self.last_id = self.last_id + 1;
                            entry.insert(self.last_id);
                            format!("{}", generate_jwt(self.last_id))
                        }
                        Entry::Occupied(entry) => {
                            format!("{}", entry.get())
                        }
                    }
                Message::Logout => "bleh".to_string(),
            }
        }
    }
}

/// JWT ///

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: u32,
    exp: u128
}

fn generate_jwt(user_id: u32) -> String {
    let my_claims = Claims {
        sub: user_id,
        exp: SystemTime::now()
                .checked_add(Duration::new(3600, 0)).unwrap()
                .duration_since(UNIX_EPOCH).unwrap()
                .as_millis()
    };

    match encode(&Header::default(), &my_claims, &EncodingKey::from_secret("secret".as_ref())) {
        Ok(token) => token,
        Err(_) => "geh".to_string()
    }
}