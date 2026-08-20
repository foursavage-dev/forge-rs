use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("ðŸ¦€ Rust Service starting on port 8080...");
    
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("âœ… Rust Service listening on 127.0.0.1:8080");
    
    loop {
        let (mut socket, _) = listener.accept().await?;
        
        tokio::spawn(async move {
            let mut buf = [0; 1024];
            
            loop {
                let n = match socket.read(&mut buf).await {
                    Ok(n) if n == 0 => return,
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("Failed to read from socket: {}", e);
                        return;
                    }
                };
                
                let response = format!("Rust Service received: {}", String::from_utf8_lossy(&buf[..n]));
                if let Err(e) = socket.write_all(response.as_bytes()).await {
                    eprintln!("Failed to write to socket: {}", e);
                    return;
                }
            }
        });
    }
}
