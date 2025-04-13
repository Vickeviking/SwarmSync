use std::process::exit;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};

#[tokio::main]
async fn main() -> io::Result<()> {
    exit(0);
}
