use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> io::Result<()> {
    // Connect to the server
    let stream = TcpStream::connect("127.0.0.1:5106").await?;
    let (reader, mut writer) = stream.into_split();

    let mut reader = BufReader::new(reader);
    let mut input = String::new();

    // Spawn a task to continuously read messages from the server
    tokio::spawn(async move {
        let mut server_response = String::new();
        while reader.read_line(&mut server_response).await.is_ok() {
            if server_response.is_empty() {
                break;
            }
            print!("{}", server_response);
            server_response.clear();
        }
    });

    // Read user input and send it to the server
    let mut stdin_reader = BufReader::new(io::stdin());
    while stdin_reader.read_line(&mut input).await.is_ok() {
        if input.trim() == "exit" {
            break;
        }
        writer.write_all(input.as_bytes()).await?;
        writer.flush().await?;
        input.clear();
    }

    Ok(())
}
