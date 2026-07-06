use music::serverstruct::Server_Room;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio::sync::broadcast;

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

    //the current amount of connections
    let connections = 0;
    // builds the server room
    let mut room = Server_Room::new_room(entry_code);
    // the arc room
    let room = Arc::new(Mutex::new(room));

    // Bind the listening socket. .await because binding/listening setup
    // can be async under the hood; .unwrap() panics if the port's taken.

    //anyone on lan can join
    let listener = TcpListener::bind("0.0.0.0:5000").await.unwrap();

    // Create a broadcast channel. Anything sent into `tx` gets delivered
    // to every active `rx` (receiver) that's subscribed.
    // The `100` is the channel's buffer size (how many unread messages
    // it holds per-subscriber before old ones get dropped for a slow reader).
    // We bind the initial receiver to `_rx` just so the channel isn't
    // immediately closed (a broadcast channel needs at least one receiver
    // reference alive at all times, even if we don't use this one directly).
    let (tx, _rx) = broadcast::channel::<String>(100);

    // Main accept loop: waits for new TCP connections forever.
    loop {
        // Accept a new client connection. .await suspends this task
        // (not the whole program) until a client connects.
        // `addr` is the client's SocketAddr - we need it as the key
        // for tracking them in the room's HashMap.
        let (socket, addr) = listener.accept().await.unwrap();

        // Clone the Arc (cheap - just a pointer + refcount bump) so this
        // task gets its own handle to the SAME underlying room.
        let room = Arc::clone(&room);

        // Each client task needs its own handle to send into the channel,
        // and its own subscription to receive from it.
        // tx.clone() is cheap - it's just cloning a handle, not the data.
        let tx = tx.clone();
        let mut rx = tx.subscribe(); // new independent receiver for this client

        // Spawn an independent async task for this client.
        // `move` transfers ownership of socket, room, tx, rx, addr into the
        // task so it can outlive this loop iteration.
        tokio::spawn(async move {
            // Split the socket into separate read/write halves so we can
            // read and write to the same connection independently/concurrently.
            let (reader, mut writer) = socket.into_split();

            // Wrap the reader so we can read line-by-line asynchronously.
            let mut reader = BufReader::new(reader).lines();

            // Step 1: ask for the code. This works for both regular users and
            // the music bot - the code they send determines which path below
            // they take.
            let _ = writer.write_all(b"Enter code: \n").await;
            let code = match reader.next_line().await {
                Ok(Some(code)) => code,
                _ => return, // disconnected before sending anything
            };

            // Lock briefly just to check the code against both possible
            // valid codes, then release - no .await happens while locked.
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

            // `match` is an expression - it can produce a value, not just run
            // side effects. Here every arm either `return`s early (rejected,
            // this task ends) or ends with a (name, is_bot) tuple. Whichever
            // arm runs, its tuple becomes the value bound to `(name, is_bot)`
            // below. This is what lets `name` survive past this whole block,
            // instead of being trapped inside a single match arm's scope.
            let (name, is_bot) = match auth {
                // Neither code matched - reject and close the connection.
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
                    }; // room_guard drops here, lock released

                    if !accepted {
                        // Tell them why, then let `writer`/`socket` drop at the
                        // end of this task - that closes the connection, no
                        // explicit close() call needed.
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
                    // Step 2: ask for a name. Has to happen here (not earlier,
                    // not in the accept loop) since we need to .await a line
                    // from THIS client specifically, without blocking the
                    // accept loop from taking the next connection.
                    let _ = writer.write_all(b"Enter your name: \n").await;
                    let name = match reader.next_line().await {
                        Ok(Some(name)) => name,
                        _ => return, // disconnected before sending a name - bail out
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
                    }; // room_guard drops here, lock released

                    if !accepted {
                        let _ = writer.write_all(b"Room is full.\n").await;
                        return;
                    }

                    // announce
                    let _ = tx.send(format!("{} joined the chat", name));

                    // This arm's value: their chosen name, and is_bot = false.
                    (name, false)
                }
            };

            // `name` and `is_bot` are now available for the rest of this task,
            // since they came out of the match as its result rather than being
            // scoped to a single arm.

            // Per-client loop: handles both "client sent something"
            // and "someone else broadcast something" at the same time.
            loop {
                // select! waits on multiple async operations simultaneously
                // and runs whichever branch becomes ready first.
                tokio::select! {
                    // Branch 1: did OUR client (user or bot) send a line?
                    line = reader.next_line() => {
                        match line {
                            // Forward it to everyone, prefixed with their name.
                            // (Bot messages will show up as "music_bot: ...".)
                            Ok(Some(line)) => {
                                let _ = tx.send(format!("{}: {}", name, line));
                            }
                            // Ok(None) = client closed the connection cleanly.
                            // Err(_) = read error.
                            // Either way, stop this client's task.
                            _ => break,
                        }
                    }

                    // Branch 2: did the broadcast channel deliver a message
                    // (from us or any other client/bot)?
                    msg = rx.recv() => {
                        if let Ok(msg) = msg {
                            // Write it out to THIS connection's socket.
                            let _ = writer.write_all(msg.as_bytes()).await;
                            let _ = writer.write_all(b"\n").await;
                        }
                        // If Err, the channel lagged or closed; ignored here.
                    }
                }
            }

            // Loop exited - this connection ended. Remove them from whichever
            // registry they actually joined (bot slot vs regular user slot),
            // then announce it to everyone still connected.
            if is_bot {
                room.lock().unwrap().remove_bot(&addr);
            } else {
                room.lock().unwrap().remove_user(&addr);
            }
            let _ = tx.send(format!("{} left the chat", name));
        });
        // Loop back around to accept the next client. This spawned task
        // keeps running independently in the background.
    }
}
