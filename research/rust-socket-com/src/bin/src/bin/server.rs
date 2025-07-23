use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, Shutdown, SocketAddr, TcpListener};

fn main() {
    let port_number = 12345;
    let listener = TcpListener::bind(SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
        port_number,
    ))
    .expect("Failed to bind to address");

    println!("Listening for connections on port {}", port_number);

    match listener.accept() {
        Ok((mut socket, addr)) => {
            println!("Accepted connection from {:?}", addr);

            let mut buffer = [0; 1024];
            match socket.read(&mut buffer) {
                Ok(bytes_read) => {
                    let message = String::from_utf8_lossy(&buffer[..bytes_read]);
                    println!("Received message is \"{}\"", message);
                }
                Err(e) => {
                    eprintln!("Error reading from client: {}", e);
                    std::process::exit(1);
                }
            }

            let response = "Hello from server";
            if let Err(e) = socket.write_all(response.as_bytes()) {
                eprintln!("Error sending message to client: {}", e);
                std::process::exit(1);
            } else {
                println!("Message sent to client");
            }

            // Close the connection.
            socket.shutdown(Shutdown::Both).expect("Shutdown failed");
        }
        Err(e) => {
            eprintln!("Accept error: {}", e);
            std::process::exit(1);
        }
    }
}
