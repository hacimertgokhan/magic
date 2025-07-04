use std::sync::Arc;
use crate::config::Config;
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::RwLock;
use crate::executor::{execute_command, executor_command_string_parser};

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
        "tcp" => start_tcp(addr).await,
        "udp" => start_udp(addr).await,
        "reflect" => start_reflect(Arc::clone(&config)).await,
        proto => {
            eprintln!("Unknow protocol: {}. 'tcp', 'udp' or 'reflect'", proto);
            Ok(())
        }
    }.unwrap_or_else(|e| {
        eprintln!("Error while starting: {:?}", e);
    });
}

async fn start_tcp(addr: String) -> tokio::io::Result<()> {
    let listener = TcpListener::bind(&addr).await?;
    println!("Listening TCP {}", addr);

    let store = Arc::new(RwLock::new(std::collections::HashMap::<String, String>::new()));

    loop {
        let (mut socket, remote) = listener.accept().await?;
        println!("New Connection: {}", remote);
        let store_clone = Arc::clone(&store);

        tokio::spawn(async move {
            let mut buf = [0u8; 1024];
            match socket.read(&mut buf).await {
                Ok(n) if n > 0 => {
                    println!("Received: {}", String::from_utf8_lossy(&buf[..n]));

                    let input = String::from_utf8_lossy(&buf[..n]).to_string();
                    let response = executor_command_string_parser(store_clone, input).await;

                    if let Err(e) = socket.write_all(response.as_bytes()).await {
                        eprintln!("Response cannot sended: {:?}", e);
                    }
                }
                _ => println!("Error on read buffers."),
            }
        });

    }
}

async fn start_udp(addr: String) -> tokio::io::Result<()> {
    let socket = UdpSocket::bind(&addr).await?;
    println!("Listening UDP {}", addr);

    let mut buf = [0u8; 1024];
    loop {
        let (n, src) = socket.recv_from(&mut buf).await?;
        println!("📨 [{}] Received: {}", src, String::from_utf8_lossy(&buf[..n]));
        socket.send_to(b"Welcome to magic!", &src).await?;
    }
}

async fn start_reflect(config: Arc<Config>) -> tokio::io::Result<()> {
    let addr = format!("{}:{}", config.server.bind_address, config.server.port);
    let targets = config.reflect.targets.clone();

    let listener = TcpListener::bind(&addr).await?;
    println!("Listening Reflect {}", addr);

    loop {
        let (mut socket, _) = listener.accept().await?;
        let targets = targets.clone();

        tokio::spawn(async move {
            let mut buffer = [0u8; 1024];

            match socket.read(&mut buffer).await {
                Ok(n) if n > 0 => {
                    let request = &buffer[..n];
                    println!("Request received: {:?}", String::from_utf8_lossy(request));

                    let mut responses = Vec::new();

                    for target_addr in targets.iter() {
                        match TcpStream::connect(target_addr).await {
                            Ok(mut target_stream) => {
                                if target_stream.write_all(request).await.is_ok() {
                                    let mut resp_buf = [0u8; 1024];
                                    if let Ok(n) = target_stream.read(&mut resp_buf).await {
                                        responses.push(String::from_utf8_lossy(&resp_buf[..n]).to_string());
                                    }
                                }
                            }
                            Err(e) => {
                                responses.push(format!("Connection error: {}", e));
                            }
                        }
                    }

                    let full_response = responses.join("\n---\n");
                    let _ = socket.write_all(full_response.as_bytes()).await;
                }
                _ => println!("There is something wrong about request or connection."),
            }
        });
    }
}
