use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() {
    let stream = TcpStream::connect("127.0.0.1:5000").await.unwrap();
    println!("Connected!, whatever u type next will be your name. Press ctrl+c to leave");

    // Split the socket into read/write halves, same as the server.
    let (reader, mut writer) = stream.into_split();
    let mut server_reader = BufReader::new(reader).lines();

    // Reader for the user's own keyboard input (stdin).
    let stdin = tokio::io::stdin();
    let mut stdin_reader = BufReader::new(stdin).lines();

    loop {
        tokio::select! {
            // Branch 1: server sent us a line (someone's chat message)
            line = server_reader.next_line() => {
                match line {
                    //The server successfully sent a complete text line
                    Ok(Some(line)) => println!("{}", line),
                    _ => {
                        println!("Disconnected from server.");
                        break;
                    }
                }
            }

            // Branch 2: user typed something and hit enter
            line = stdin_reader.next_line() => {
                match line {
                    Ok(Some(line)) => {
                        let _ = writer.write_all(line.as_bytes()).await;
                        let _ = writer.write_all(b"\n").await;
                    }//if user wants to quit
                    _ => break,
                }
            }
        }
    }
}
