use music::serverstruct::Server_Room;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio::sync::broadcast;

#[derive(Clone)]

enum AuthType {
    User,
    MusicBot,
}
fn code_gen() -> String {
    let entry_code = Server_Room::generate_code(9);
    println!("The room code is {}", entry_code);
    entry_code
}
// This macro wraps main in a tokio runtime so we can use .await here.
#[tokio::main]
async fn main() {
    // room code join gen

    let entry_code = code_gen();

    // builds the server room
    let room = Server_Room::new_room(entry_code);
    // the arc room
    let room = Arc::new(Mutex::new(room));

    //anyone on lan can join
    let listener = TcpListener::bind("0.0.0.0:5000").await.unwrap();

    // Create a broadcast channel. Anything sent into tx gets delivered
    // to every active `rx` receiver
    // The `100` is the channel's buffer size
    let (tx, _rx) = broadcast::channel::<String>(100);

    // Main accept loop: waits for new TCP connections forever.
    loop {
        //probably the most confusing part

        // Accept a new client connection. await pauses the program until someone joins
        let (socket, addr) = listener.accept().await.unwrap();

        // Clone the room with arc so it can handle multiple people joining
        let room = Arc::clone(&room);

        // Each client task needs its own handle to send into the channel, and its own subscription to receive from it so i use clone so each client can have one
        let tx = tx.clone();
        let mut rx = tx.subscribe(); // new independent receiver for this client

        // Spawn an independent async task for the person that joined

        tokio::spawn(async move {
            // Split the socket into separate read/write halves so we can
            // read and write to the same connection independently/concurrently.
            let (reader, mut writer) = socket.into_split();

            // Wrap the reader so we can read line-by-line asynchronously.
            let mut reader = BufReader::new(reader).lines();

            // Step 1: ask for the code.
            let _ = writer.write_all(b"Enter code: \n").await;
            let code = match reader.next_line().await {
                Ok(Some(code)) => code,
                _ => return, // disconnected before sending anything
            };

            // Locks the process for a little bit so it can validate who is entering and if there is room
            let auth = {
                let room = room.lock().unwrap();

                if room.code_is_valid(&code) {
                    Some(AuthType::User)
                } else if room.music_code_valid(&code) {
                    Some(AuthType::MusicBot)
                } else {
                    None
                }
            };

            // handles what/who is joning and assigns name and if bool if they are bot or not
            let (name, is_bot) = match auth {
                // Neither code matched, reject and close the connection.
                None => {
                    let _ = writer.write_all(b"Invalid code.\n").await;
                    return;
                }

                // Music code valid register as the bot
                Some(AuthType::MusicBot) => {
                    // check if bot has spot
                    let accepted = {
                        let mut room_guard = room.lock().unwrap();
                        if room_guard.has_space_music_bot() {
                            room_guard.add_bot(addr, "music_bot".to_string());
                            true
                        } else {
                            false
                        }
                    };

                    if !accepted {
                        let _ = writer.write_all(b"Bot room is full.\n").await;
                        return;
                    }

                    // giving th bot a fixed name
                    let name = "music_bot".to_string();

                    // annouce the bot is here
                    let _ = tx.send(format!("{} joined the chat", name));

                    // This arm's value: bot's fixed name, and is_bot = true
                    // so cleanup later knows to call remove_bot, not remove_user.
                    (name, true)
                }

                // for normal user
                Some(AuthType::User) => {
                    // asks for name
                    let _ = writer.write_all(b"Enter your name: \n").await;
                    let name = match reader.next_line().await {
                        Ok(Some(name)) => name,
                        _ => return, // disconnected before sending a name
                    };

                    // check if user can fit
                    let accepted = {
                        let mut room_guard = room.lock().unwrap();
                        if room_guard.has_space() {
                            room_guard.add_user(addr, name.clone());
                            true
                        } else {
                            false
                        }
                    };

                    if !accepted {
                        let _ = writer.write_all(b"Room is full.\n").await;
                        return;
                    }

                    // announce
                    let _ = tx.send(format!("{} joined the chat", name));

                    // user name and false cause they arent a bot
                    (name, false)
                }
            };

            //The group chat part
            loop {
                // from my understanding, select! waits on multiple async operations simultaneously and runs whichever branch becomes ready first.
                tokio::select! {
                    // Branch 1: was a line sent?
                    line = reader.next_line() => {
                        match line {
                            // Forward it to everyone  with their name.
                            // EXample: (Bot messages will show up as "music_bot: ...".)
                            Ok(Some(line)) => {
                                let _ = tx.send(format!("{}: {}", name, line));
                            }
                            //probably not the best way but if any issue stop this client's task
                            _ => break,
                        }
                    }

                    // Branch 2: did the broadcast channel deliver a message

                    msg = rx.recv() => {
                        if let Ok(msg) = msg {
                            // Write it out to connection's socket.
                            let _ = writer.write_all(msg.as_bytes()).await;
                            let _ = writer.write_all(b"\n").await;
                        }
                        // If Err, the channel lagged or closed. ignored here.

                    }
                }
            }

            // Loop exited, this connection ended. Remove them from whichever registry they actually joined and then announce it to everyone still connected.
            if is_bot {
                room.lock().unwrap().remove_bot(&addr);
            } else {
                room.lock().unwrap().remove_user(&addr);
            }
            let _ = tx.send(format!("{} left the chat", name));
        });
    }
}
