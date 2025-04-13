use std::process::exit;

use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

pub mod grpc;

#[tokio::main]
async fn main() -> io::Result<()> {
    exit(0);
}
