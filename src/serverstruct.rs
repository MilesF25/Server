use rand::RngExt;
use std::collections::HashMap;
use std::net::SocketAddr;
pub struct Server_Room {
    //to enter
    entry_code: String,

    //current users
    users: HashMap<SocketAddr, String>,

    // music bot
    music_bot_token: String,
    current_bots: HashMap<SocketAddr, String>,
}

impl Server_Room {
    //makes code for server connect, this will run before the server actually starts up
    pub fn new_room(entry_code: String) -> Self {
        Server_Room {
            entry_code,
            users: HashMap::new(),

            music_bot_token: "E3ICY0BV5KAFKU45Y7_musicbot".to_string(),
            current_bots: HashMap::new(),
        }
    }
    //joining code
    pub fn generate_code(len: usize) -> String {
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        let mut rng = rand::rng();

        (0..len)
            .map(|_| {
                let idx = rng.random_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }

    pub fn has_space_music_bot(&self) -> bool {
        // checks if len of bots is more than one. if it is then false
        self.current_bots.len() < 1
    }

    pub fn has_space(&self) -> bool {
        // Returns true if under 4, false if 4 or more
        self.users.len() < 4
    }

    // when someone joins
    pub fn add_user(&mut self, addr: SocketAddr, name: String) {
        self.users.insert(addr, name);
    }

    // add music bot

    pub fn add_bot(&mut self, addr: SocketAddr, name: String) {
        self.current_bots.insert(addr, name);
    }

    //  when someone leaves
    pub fn remove_user(&mut self, addr: &SocketAddr) -> Option<String> {
        self.users.remove(addr) // returns their name so it can be announced
    }

    // removes the bot
    pub fn remove_bot(&mut self, addr: &SocketAddr) -> Option<String> {
        self.current_bots.remove(addr)
    }

    // code check
    pub fn code_is_valid(&self, code: &str) -> bool {
        code == self.entry_code
    }

    // music bot code_check
    pub fn music_code_valid(&self, music_code: &str) -> bool {
        music_code == self.music_bot_token
    }
}
