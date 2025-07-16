use std::net::SocketAddr;
use std::collections::HashMap;
use std::sync::Arc;

use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio::task::LocalSet;
use tokio_tungstenite::{accept_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};

type ClientId = SocketAddr;
type ClientSender = tokio::sync::mpsc::UnboundedSender<Message>;
type Clients = Arc<Mutex<HashMap<ClientId, ClientSender>>>;

#[derive(Debug, Clone)]
enum ChatEvent {
    ClientConnected(ClientId),
    ClientDisconnected(ClientId),
    MessageReceived { from: ClientId, content: String },
    NameSet { client: ClientId, name: String },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr: SocketAddr = "127.0.0.1:8080".parse()?;
    println!("Starting WebSocket chat server (tokio-tungstenite) on {}", addr);
    
    // WebSocket operations require LocalSet for proper execution with DFIR
    LocalSet::new().run_until(async {
        let listener = TcpListener::bind(addr).await?;
        println!("Chat server bound to: {}", addr);
        
        let clients: Clients = Arc::new(Mutex::new(HashMap::new()));
        
        // Create channels for DFIR integration
        let (event_sender, mut event_receiver) = tokio::sync::mpsc::unbounded_channel();
        let (broadcast_sender, mut broadcast_receiver) = tokio::sync::mpsc::unbounded_channel();
        
        // Start DFIR message processing
        let dfir_clients = clients.clone();
        tokio::task::spawn_local(async move {
            run_dfir_chat_logic(event_receiver, broadcast_sender, dfir_clients).await;
        });
        
        // Start broadcast handler
        let broadcast_clients = clients.clone();
        tokio::task::spawn_local(async move {
            while let Some((message, sender_id)) = broadcast_receiver.recv().await {
                broadcast_to_clients(&broadcast_clients, message, Some(sender_id)).await;
            }
        });
        
        println!("Chat server running... Press Ctrl+C to stop");
        
        // Accept connections
        while let Ok((stream, addr)) = listener.accept().await {
            let clients = clients.clone();
            let event_sender = event_sender.clone();
            
            tokio::task::spawn_local(async move {
                if let Err(e) = handle_connection(stream, addr, clients, event_sender).await {
                    eprintln!("Error handling connection from {}: {}", addr, e);
                }
            });
        }
        
        Ok::<(), Box<dyn std::error::Error>>(())
    }).await
}

async fn handle_connection(
    stream: TcpStream,
    addr: SocketAddr,
    clients: Clients,
    event_sender: tokio::sync::mpsc::UnboundedSender<ChatEvent>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("New connection from: {}", addr);
    
    // Perform WebSocket handshake
    let ws_stream = accept_async(stream).await?;
    println!("WebSocket handshake completed for: {}", addr);
    
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    
    // Create channel for this client
    let (client_sender, mut client_receiver) = tokio::sync::mpsc::unbounded_channel();
    
    // Register client
    {
        let mut clients_lock = clients.lock().await;
        clients_lock.insert(addr, client_sender);
    }
    
    // Notify DFIR about new connection
    let _ = event_sender.send(ChatEvent::ClientConnected(addr));
    
    // Send welcome message
    let welcome_msg = Message::Text("Welcome to the DFIR WebSocket chat server!".to_string());
    if let Err(e) = ws_sender.send(welcome_msg).await {
        eprintln!("Failed to send welcome message to {}: {}", addr, e);
    }
    
    // Handle client messages and outgoing messages concurrently
    let event_sender_clone = event_sender.clone();
    let receive_task = tokio::spawn(async move {
        while let Some(msg_result) = ws_receiver.next().await {
            match msg_result {
                Ok(msg) => {
                    match msg {
                        Message::Text(text) => {
                            println!("Received from {}: {}", addr, text);
                            let _ = event_sender_clone.unbounded_send(ChatEvent::MessageReceived {
                                from: addr,
                                content: text,
                            });
                        }
                        Message::Close(_) => {
                            println!("Client {} sent close frame", addr);
                            break;
                        }
                        Message::Ping(data) => {
                            // Echo pong back
                            // This will be handled by the send task
                        }
                        _ => {
                            // Handle other message types if needed
                        }
                    }
                }
                Err(e) => {
                    eprintln!("WebSocket error from {}: {}", addr, e);
                    break;
                }
            }
        }
    });
    
    let send_task = tokio::spawn(async move {
        while let Some(message) = client_receiver.recv().await {
            if let Err(e) = ws_sender.send(message).await {
                eprintln!("Failed to send message to {}: {}", addr, e);
                break;
            }
        }
    });
    
    // Wait for either task to complete
    tokio::select! {
        _ = receive_task => {},
        _ = send_task => {},
    }
    
    // Cleanup: remove client and notify DFIR
    {
        let mut clients_lock = clients.lock().await;
        clients_lock.remove(&addr);
    }
    
    let _ = event_sender.unbounded_send(ChatEvent::ClientDisconnected(addr));
    println!("Client {} disconnected", addr);
    
    Ok(())
}

async fn run_dfir_chat_logic(
    mut event_receiver: tokio::sync::mpsc::UnboundedReceiver<ChatEvent>,
    broadcast_sender: tokio::sync::mpsc::UnboundedSender<(Message, ClientId)>,
    _clients: Clients,
) {
    let mut user_names: HashMap<ClientId, String> = HashMap::new();
    
    println!("DFIR chat logic running...");
    
    // Simple message processing without DFIR flow for now
    while let Some(event) = event_receiver.recv().await {
        match event {
            ChatEvent::ClientConnected(client_id) => {
                println!("DFIR: Client {} connected", client_id);
                let message = Message::Text(format!("Client {} joined the chat", client_id));
                let _ = broadcast_sender.send((message, client_id));
            }
            ChatEvent::ClientDisconnected(client_id) => {
                println!("DFIR: Client {} disconnected", client_id);
                user_names.remove(&client_id);
                let message = Message::Text(format!("Client {} left the chat", client_id));
                let _ = broadcast_sender.send((message, client_id));
            }
            ChatEvent::MessageReceived { from, content } => {
                if content.starts_with("/name ") {
                    let name = content[6..].trim().to_string();
                    println!("DFIR: Client {} set name to: {}", from, name);
                    user_names.insert(from, name.clone());
                    let message = Message::Text(format!("You are now known as {}", name));
                    let _ = broadcast_sender.send((message, from));
                } else if content.starts_with("/") {
                    let message = Message::Text("Unknown command. Available commands: /name <your_name>".to_string());
                    let _ = broadcast_sender.send((message, from));
                } else {
                    // Regular chat message - format with user name if available
                    let sender_name = user_names.get(&from)
                        .map(|name| name.clone())
                        .unwrap_or_else(|| from.to_string());
                    let message = Message::Text(format!("{}: {}", sender_name, content));
                    let _ = broadcast_sender.send((message, from));
                }
            }
            ChatEvent::NameSet { client: _, name: _ } => {
                // Handle name setting
            }
        }
    }
}

async fn broadcast_to_clients(
    clients: &Clients,
    message: Message,
    exclude_sender: Option<ClientId>,
) {
    let clients_lock = clients.lock().await;
    for (client_id, sender) in clients_lock.iter() {
        if Some(*client_id) != exclude_sender {
            if let Err(_) = sender.send(message.clone()) {
                eprintln!("Failed to broadcast to client: {}", client_id);
            }
        }
    }
}
