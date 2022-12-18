use std::io::{BufReader, Write, BufRead};
use std::net::{TcpStream};
use std::thread;
use rand::prelude::*;

pub struct TcpClient {
    stream: TcpStream,
    ping_or_pong_chance: f64, // 0.0 - 1.0, defines the chance sending of ping or pong
    miss_chance: f64 // 0.0 - 1.0, defines the chance of sending a miss
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
    pub fn new(server_ip: &str, miss_chance: f64, server_port: u16) -> Result<TcpClient, String> {
        match TcpStream::connect(format!("{}:{}", server_ip, server_port)) {
            Ok(stream) => {
                Ok(TcpClient { stream, ping_or_pong_chance: (1.0 - miss_chance) / 2.0, miss_chance })
            },
            Err(e) => Err(format!("TcpClient: Error during connecting to server {}:{}. {:?}", server_ip, server_port, e))
        }
    }

    pub fn run(mut self) -> Result<thread::JoinHandle<()>, String> {
        match thread::Builder::new().name("client_thread".to_string()).spawn(move || {
            self.send_random_message();
            loop {
                let mut buf_reader = BufReader::new(&self.stream);
                let mut buf = String::new();
                match buf_reader.read_line(&mut buf) {
                    Ok(_) => {
                        let trimmed = buf.trim();
                        if trimmed == "GameOver" {
                            break;
                        } else {
                            self.send_random_message();
                        }
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

    fn send_random_message(&mut self) {
        let mut rng = rand::thread_rng();
        let random_number: f64 = rng.gen();
        if random_number < self.miss_chance {
            self.stream.write("Miss\n".as_bytes()).unwrap();
            // log!("Sent Miss");
        } else if random_number < self.ping_or_pong_chance + self.miss_chance {
            self.stream.write("Pong\n".as_bytes()).unwrap();
            // log!("Sent Pong");
        } else {
            self.stream.write("Ping\n".as_bytes()).unwrap();
            // log!("Sent Ping");
        }
        self.stream.flush().unwrap();
    }
}