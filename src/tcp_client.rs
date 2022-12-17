use std::io::{Read, Write};
use std::net::{TcpStream};
use std::thread;

pub struct TcpClient {
    stream: TcpStream
}

macro_rules! log {
    () => {
        println!();
    };
    ($($arg:tt)*) => {{
        println!("TcpClient: {}", format!($($arg)*));
    }};
}

impl TcpClient {
    pub fn new(server_ip: &str, server_port: u16) -> Result<TcpClient, String> {
        match TcpStream::connect(format!("{}:{}", server_ip, server_port)) {
            Ok(stream) => {
                Ok(TcpClient { stream })
            },
            Err(e) => Err(format!("TcpClient: Error during connecting to server {}:{}. {:?}", server_ip, server_port, e))
        }
    }

    pub fn run(mut self) -> Result<thread::JoinHandle<()>, String> {
        match thread::Builder::new().name("client_thread".to_string()).spawn(move || {
            self.stream.write_all("PingPong!".as_bytes()).unwrap();
            self.stream.flush().unwrap();
            log!("Sent: PingPong!");
            loop {
                let mut buffer = String::new();
                match self.stream.read_to_string(&mut buffer) {
                    Ok(_) => {
                        log!("Received: {}", buffer);
                        self.stream.write_all(buffer.as_bytes()).unwrap();
                    },
                    Err(e) => {
                        log!("Error reading from stream: {:?}", e);
                    }
                }
            }
        }) {
            Ok(join_handle) => Ok(join_handle),
            Err(e) => Err(format!("Error spawning client thread: {:?}", e))
        }
    }
}