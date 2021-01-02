use std::collections::HashMap;
use std::collections::hash_map::Entry;
use serde::{Deserialize, Serialize};
use serde_json;

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
                            format!("{}", self.last_id)
                        }
                        Entry::Occupied(entry) => {
                            format!("{}", entry.get())
                        }
                    }
                Message::Logout => "bleh".to_string(),
            }
        }
        // string
    }
}

// pub trait UserHandling {
//     fn get_user(&mut self, string: String) -> String;
// }

// impl UserHandling for Users {
//     fn get_user(&mut self, string: String) -> String {
//         let mut split = string.split(':');
//         match split.next() {
//             Option::None => "No username provided".to_string(),
//             Option::Some(user) => 
//                 match split.next() {
//                     Option::None => "No password provided".to_string(),
//                     Option::Some(password) => {
//                         let entry = self.entry((user.to_string(), password.to_string()));
//                         entry.or_insert(32);
//                         log::debug!("User {}, Password {}", user, password);
//                         "id:250,expires_at:250000".to_string()
//                     }
//             }
//         }
//     }
// }