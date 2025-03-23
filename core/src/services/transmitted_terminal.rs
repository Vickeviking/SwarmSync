use crate::enums::system::CoreEvent;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWrite, AsyncWriteExt, BufReader, WriteHalf};
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::{broadcast::Receiver, mpsc::UnboundedSender, Mutex};

pub struct TransmittedTerminal {}

impl TransmittedTerminal {
    pub async fn init(
        mut core_event_rx: Receiver<CoreEvent>,
        core_event_manip_tx: UnboundedSender<CoreEvent>,
    ) {
        let listener = TcpListener::bind("0.0.0.0:5106")
            .await
            .expect("Failed to bind port");
        let active_users = Arc::new(Mutex::new(HashMap::new()));

        loop {
            tokio::select! {
                Ok((socket, addr)) = listener.accept() => {
                    println!("New connection from: {}", addr);
                    let users = Arc::clone(&active_users);
                    let core_event_tx = core_event_manip_tx.clone();
                    tokio::spawn(Self::handle_client(socket, addr.to_string(), users, core_event_tx));
                },

                event = core_event_rx.recv() => {
                    match event {
                        Ok(CoreEvent::Startup) => println!("TransmittedTerminal: Startup event received."),
                        Ok(CoreEvent::Restart) => println!("TransmittedTerminal: Restart event received."),
                        Ok(CoreEvent::Shutdown) => {
                            println!("TransmittedTerminal: Shutdown event received. Stopping...");
                            break;
                        }
                        Err(_) => {
                            println!("TransmittedTerminal: Channel closed. Exiting...");
                            break;
                        }
                    }
                }
            }
        }
    }

    // Update function to accept WriteHalf instead of Pin<&mut dyn AsyncWrite>
    async fn clean_terminal(writer: &mut WriteHalf<TcpStream>) {
        let clear_sequence = vec![0x1B, b'[', 0x32, b'J', 0x1B, b'[', 0x31, b';', 0x31, b'H'];
        writer.write_all(&clear_sequence).await.unwrap();
    }

    async fn handle_client(
        socket: TcpStream,
        addr: String,
        active_users: Arc<Mutex<HashMap<String, String>>>,
        core_event_tx: UnboundedSender<CoreEvent>,
    ) {
        let (reader, mut writer) = tokio::io::split(socket);
        let mut reader = BufReader::new(reader);

        // Step 1: Ask for login
        writer.write_all(b"Enter your username: ").await.unwrap();
        let mut username = String::new();
        if reader.read_line(&mut username).await.is_err() {
            println!("Client {} disconnected before login.", addr);
            return;
        }
        let username = username.trim().to_string();

        {
            let mut users = active_users.lock().await;
            users.insert(addr.clone(), username.clone());
        }

        println!("User '{}' logged in from {}", username, addr);
        writer.write_all(
        b"\nWelcome! Choose an option:\n1) Startup\n2) Shutdown\n3) Restart\n4) Disconnect\n",
    )
    .await
    .unwrap();

        // Step 2: Interactive menu loop
        let mut buffer = String::new();
        let mut menu_state = "main"; // Keeps track of current menu

        while let Ok(n) = reader.read_line(&mut buffer).await {
            if n == 0 {
                break;
            }

            let choice = buffer.trim();

            match menu_state {
                "main" => match choice {
                    "1" => {
                        writer
                            .write_all(
                                b"Startup selected! Choose:\n1) Normal Startup\n2) Safe Mode\n",
                            )
                            .await
                            .unwrap();
                        menu_state = "startup"; // Move to Startup submenu
                    }
                    "2" => {
                        writer.write_all(b"Shutdown initiated.\n").await.unwrap();
                        let _ = core_event_tx.send(CoreEvent::Shutdown);
                        break;
                    }
                    "3" => {
                        Self::clean_terminal(&mut writer).await;
                        writer.write_all(b"Restarting...\n").await.unwrap();
                        let _ = core_event_tx.send(CoreEvent::Restart);
                    }
                    "4" => {
                        writer.write_all(b"Goodbye!\n").await.unwrap();
                        break;
                    }
                    _ => {
                        writer
                            .write_all(b"Invalid option. Try again:\n")
                            .await
                            .unwrap();
                    }
                },

                "startup" => {
                    match choice {
                        "1" => {
                            writer
                                .write_all(b"Normal Startup selected!\n")
                                .await
                                .unwrap();

                            // Use the clean_terminal function to clear the screen
                            Self::clean_terminal(&mut writer).await;
                            writer
                                .write_all(b"Returning to main menu...\n")
                                .await
                                .unwrap();
                            menu_state = "main"; // Go back to main menu
                        }
                        "2" => {
                            writer.write_all(b"Safe Mode selected!\n").await.unwrap();

                            // Use the clean_terminal function to clear the screen
                            Self::clean_terminal(&mut writer).await;
                            writer
                                .write_all(b"Returning to main menu...\n")
                                .await
                                .unwrap();
                            menu_state = "main"; // Go back to main menu
                        }
                        _ => {
                            writer
                            .write_all(b"Invalid option. Choose again:\n1) Normal Startup\n2) Safe Mode\n")
                            .await
                            .unwrap();
                        }
                    }
                }

                _ => {
                    writer
                        .write_all(b"Invalid state. Resetting to main menu...\n")
                        .await
                        .unwrap();
                    menu_state = "main";
                }
            }

            // Always show menu unless in a nested menu
            if menu_state == "main" {
                writer
                    .write_all(
                        b"\nMain Menu:\n1) Startup\n2) Shutdown\n3) Restart\n4) Disconnect\n",
                    )
                    .await
                    .unwrap();
            }

            buffer.clear();
        }

        // Step 3: Handle disconnection
        {
            let mut users = active_users.lock().await;
            users.remove(&addr);
        }
        println!("User '{}' disconnected from {}", username, addr);
    }
}
