use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() {
    let stream = TcpStream::connect("127.0.0.1:5000").await.unwrap();

    // should do a handshake with server. will say that this is bot, server acks and then preps for music then gives ok, bot sends music audio

    // will need threading so human can skip music or maybe even change playlists

    //change server to have 5 spots but 1 for client

    // Split the socket into read/write halves, same as the server.
    let (reader, mut writer) = stream.into_split();
    let mut server_reader = BufReader::new(reader).lines();
    // have it send:  "E3ICY0BV5KAFKU45Y7_musicbot". to server so it knows its a bot
}
