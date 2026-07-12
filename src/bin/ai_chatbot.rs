use dotenv::dotenv;
use gemini_rs;
use std::env;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let api_key = env::var("MY_API_KEY").expect("API_KEY not found");
    let server_addr = "127.0.0.1:5000";
    let bot_code = "E3ICY0BV5KAFKU45Y7_musicbot";

    let stream = TcpStream::connect(server_addr).await?;
    let (reader, mut writer) = stream.into_split();
    let mut server_reader = BufReader::new(reader).lines();

    writer
        .write_all(format!("{}\n", bot_code).as_bytes())
        .await?;
    println!("Chatbot connected to server...");

    // Create the client once outside the loop
    let client = gemini_rs::Client::new(api_key);

    while let Ok(Some(line)) = server_reader.next_line().await {
        // Ignore messages sent by the bot itself to prevent infinite loops
        if line.starts_with("music_bot:") {
            continue;
        }

        let lower_line = line.to_lowercase();

        // Use a slightly strict trigger to avoid accidental activation
        if let Some(pos) = lower_line.find(": gemini") {
            let prompt = line[pos + 9..].trim();

            if !prompt.is_empty() {
                println!("Asking Gemini: {}", prompt);

                match call_google_ai(&client, prompt).await {
                    Ok(response) => {
                        let chat_msg = format!("Gemini-Bot: {}\n", response);
                        let _ = writer.write_all(chat_msg.as_bytes()).await;
                    }
                    Err(e) => {
                        eprintln!("API Error: {}", e);
                    }
                }
            }
        }
    }
    Ok(())
}

async fn call_google_ai(
    client: &gemini_rs::Client,
    prompt: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut chat = client.chat("gemini-2.0-flash");
    let response = chat.send_message(prompt).await?;
    Ok(response.to_string())
}
