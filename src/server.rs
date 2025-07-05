use std::sync::Arc;
use crate::config::{Config, UserCredentials};
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::RwLock;
use crate::executor::{execute_command, executor_command_string_parser};
use std::collections::HashMap;

pub async fn start(config: Config) {
    println!("Server started at: {}:{} [{}]",
             config.server.bind_address,
             config.server.port,
             config.server.protocol.clone().unwrap_or("tcp".into()).to_uppercase()
    );

    let protocol = config.server.protocol.clone().unwrap_or("tcp".to_string());
    let addr = format!("{}:{}", config.server.bind_address, config.server.port);

    let config = Arc::new(config);

    match protocol.to_lowercase().as_str() {
        "tcp" => start_tcp(addr, Arc::clone(&config)).await,
        "udp" => start_udp(addr, Arc::clone(&config)).await,
        "reflect" => start_reflect(Arc::clone(&config)).await,
        proto => {
            eprintln!("Unknown protocol: {}. 'tcp', 'udp' or 'reflect'", proto);
            Ok(())
        }
    }.unwrap_or_else(|e| {
        eprintln!("Error while starting: {:?}", e);
    });
}

async fn start_tcp(addr: String, config: Arc<Config>) -> tokio::io::Result<()> {
    let listener = TcpListener::bind(&addr).await?;
    println!("Listening TCP {}", addr);

    let store = Arc::new(RwLock::new(std::collections::HashMap::<String, String>::new()));

    loop {
        let (mut socket, remote) = listener.accept().await?;
        println!("New Connection: {}", remote);
        let store_clone = Arc::clone(&store);
        let config_clone = Arc::clone(&config);

        tokio::spawn(async move {
            let mut buf = [0u8; 1024];
            match socket.read(&mut buf).await {
                Ok(n) if n > 0 => {
                    let input = String::from_utf8_lossy(&buf[..n]).to_string();
                    println!("Received: {}", input);

                    // Check for authentication
                    if input.trim().starts_with("AUTH ") {
                        let parts: Vec<&str> = input.trim().split_whitespace().collect();
                        if parts.len() >= 3 {
                            let username = parts[1];
                            let password = parts[2];

                            if username == config_clone.user.username && password == config_clone.user.password {
                                let _ = socket.write_all(b"AUTH OK\n").await;

                                // Continue to read the actual command
                                let mut cmd_buf = [0u8; 1024];
                                if let Ok(n) = socket.read(&mut cmd_buf).await {
                                    if n > 0 {
                                        let command = String::from_utf8_lossy(&cmd_buf[..n]).to_string();
                                        let response = executor_command_string_parser(store_clone, command).await;
                                        let _ = socket.write_all(response.as_bytes()).await;
                                    }
                                }
                            } else {
                                let _ = socket.write_all(b"AUTH FAILED\n").await;
                            }
                        } else {
                            let _ = socket.write_all(b"AUTH FAILED\n").await;
                        }
                    } else {
                        // Direct command without auth (for backward compatibility)
                        let response = executor_command_string_parser(store_clone, input).await;
                        if let Err(e) = socket.write_all(response.as_bytes()).await {
                            eprintln!("Response cannot be sent: {:?}", e);
                        }
                    }
                }
                _ => println!("Error on read buffers."),
            }
        });
    }
}

async fn start_udp(addr: String, config: Arc<Config>) -> tokio::io::Result<()> {
    let socket = UdpSocket::bind(&addr).await?;
    println!("Listening UDP {}", addr);

    let store = Arc::new(RwLock::new(std::collections::HashMap::<String, String>::new()));

    let mut buf = [0u8; 1024];
    loop {
        let (n, src) = socket.recv_from(&mut buf).await?;
        let input = String::from_utf8_lossy(&buf[..n]).to_string();
        println!("[{}] Received: {}", src, input);

        // Check for authentication
        if input.trim().starts_with("AUTH ") {
            let parts: Vec<&str> = input.trim().split_whitespace().collect();
            if parts.len() >= 3 {
                let username = parts[1];
                let password = parts[2];

                if username == config.user.username && password == config.user.password {
                    let _ = socket.send_to(b"AUTH OK", &src).await;
                } else {
                    let _ = socket.send_to(b"AUTH FAILED", &src).await;
                }
            } else {
                let _ = socket.send_to(b"AUTH FAILED", &src).await;
            }
        } else {
            // Process command
            let store_clone = Arc::clone(&store);
            let response = executor_command_string_parser(store_clone, input).await;

            if let Err(e) = socket.send_to(response.as_bytes(), &src).await {
                eprintln!("Response cannot be sent: {:?}", e);
            }
        }
    }
}

async fn start_reflect(config: Arc<Config>) -> tokio::io::Result<()> {
    let addr = format!("{}:{}", config.server.bind_address, config.server.port);
    let targets = config.reflect.targets.clone();
    let target_users = config.user.targets.clone().unwrap_or_default();

    let listener = TcpListener::bind(&addr).await?;
    println!("Listening Reflect {}", addr);

    // Reflect store for send to functionality
    let reflect_store = Arc::new(RwLock::new(std::collections::HashMap::<String, String>::new()));

    loop {
        let (mut socket, _) = listener.accept().await?;
        let targets = targets.clone();
        let target_users = target_users.clone();
        let reflect_store_clone = Arc::clone(&reflect_store);
        let main_user = config.user.clone();

        tokio::spawn(async move {
            let mut buffer = [0u8; 1024];

            match socket.read(&mut buffer).await {
                Ok(n) if n > 0 => {
                    let request_str = String::from_utf8_lossy(&buffer[..n]).to_string();
                    println!("Request received: {}", request_str);
                    if request_str.trim().starts_with("AUTH ") {
                        let parts: Vec<&str> = request_str.trim().split_whitespace().collect();
                        if parts.len() >= 3 {
                            let username = parts[1];
                            let password = parts[2];

                            if username == main_user.username && password == main_user.password {
                                let _ = socket.write_all(b"AUTH OK\n").await;
                                let mut cmd_buf = [0u8; 1024];
                                if let Ok(n) = socket.read(&mut cmd_buf).await {
                                    if n > 0 {
                                        let command = String::from_utf8_lossy(&cmd_buf[..n]).to_string();
                                        process_reflect_command(socket, command, targets, target_users, reflect_store_clone).await;
                                    }
                                }
                            } else {
                                let _ = socket.write_all(b"AUTH FAILED\n").await;
                            }
                        } else {
                            let _ = socket.write_all(b"AUTH FAILED\n").await;
                        }
                        return;
                    }
                    process_reflect_command(socket, request_str, targets, target_users, reflect_store_clone).await;
                }
                _ => println!("There is something wrong about request or connection."),
            }
        });
    }
}

async fn process_reflect_command(
    mut socket: TcpStream,
    request_str: String,
    targets: Vec<String>,
    target_users: HashMap<String, UserCredentials>,
    reflect_store: Arc<RwLock<HashMap<String, String>>>
) {
    if request_str.trim().to_lowercase().starts_with("send to ") {
        let parts: Vec<&str> = request_str.trim().split_whitespace().collect();
        if parts.len() >= 3 {
            let target_ip = parts[2]; // "send to 127.0.0.1:8080"
            let store_read = reflect_store.read().await;
            let data_to_send = store_read.get("last_reflect_data")
                .unwrap_or(&"No data available".to_string()).clone();
            drop(store_read);
            match TcpStream::connect(target_ip).await {
                Ok(mut target_stream) => {
                    if target_stream.write_all(data_to_send.as_bytes()).await.is_ok() {
                        let response = format!("Data sent to {}", target_ip);
                        let _ = socket.write_all(response.as_bytes()).await;
                    } else {
                        let response = format!("Failed to send data to {}", target_ip);
                        let _ = socket.write_all(response.as_bytes()).await;
                    }
                }
                Err(e) => {
                    let response = format!("Connection error to {}: {}", target_ip, e);
                    let _ = socket.write_all(response.as_bytes()).await;
                }
            }
        }
        return;
    }

    let request = request_str.as_bytes();
    let mut responses = Vec::new();

    for target_addr in targets.iter() {
        match TcpStream::connect(target_addr).await {
            Ok(mut target_stream) => {
                if let Some(user_creds) = target_users.get(target_addr) {
                    let auth_msg = format!("AUTH {} {}\n", user_creds.username, user_creds.password);
                    if target_stream.write_all(auth_msg.as_bytes()).await.is_err() {
                        responses.push(format!("Authentication failed for {}", target_addr));
                        continue;
                    }
                    let mut auth_buf = [0u8; 1024];
                    if let Ok(n) = target_stream.read(&mut auth_buf).await {
                        let auth_response = String::from_utf8_lossy(&auth_buf[..n]);
                        if !auth_response.contains("OK") {
                            responses.push(format!("Authentication rejected for {}", target_addr));
                            continue;
                        }
                    }
                }
                if target_stream.write_all(request).await.is_ok() {
                    let mut resp_buf = [0u8; 1024];
                    if let Ok(n) = target_stream.read(&mut resp_buf).await {
                        let response_data = String::from_utf8_lossy(&resp_buf[..n]).to_string();
                        responses.push(response_data.clone());
                        reflect_store.write().await.insert("last_reflect_data".to_string(), response_data);
                    }
                }
            }
            Err(e) => {
                responses.push(format!("Connection error to {}: {}", target_addr, e));
            }
        }
    }

    let full_response = responses.join("\n---\n");
    let _ = socket.write_all(full_response.as_bytes()).await;
}