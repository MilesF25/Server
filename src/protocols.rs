use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMessage {
    Join { username: String },

    Chat { message: String },

    Disconnect,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerMessage {
    Welcome,

    Chat { username: String, message: String },

    UserJoined { username: String },

    UserLeft { username: String },

    Error { message: String },
}
