use serde::{Deserialize, Serialize};

#[derive(Default, Deserialize, Serialize, Debug)]
pub struct User {
    pub username: String,
}

impl User {
    pub fn key() -> String {
        "username".to_string()
    }
}
