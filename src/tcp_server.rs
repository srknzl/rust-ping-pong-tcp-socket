use std::io::{Write, BufReader, BufRead};
use std::net::{TcpListener, TcpStream, SocketAddr};
use std::sync::mpsc::{Sender};
use std::thread;
use std::time::Duration;

use crate::SLEEP_MILLIS_AFTER_SCORING;
use crate::SCORE_TO_WIN;

pub struct TcpServer {
    listener: TcpListener,
    client_counter: u64
}

struct GameState {
    client_name: String,
    client_score: u8,
    server_score: u8,
    client_sent_last_ping: bool
}

macro_rules! log {
    () => {
        println!();
    };
    ($($arg:tt)*) => {{
        println!("TcpServer: {}", format!($($arg)*));
    }};
}

impl TcpServer {
    pub fn new(ip: &str, port: u16) -> Result<TcpServer, String> {
        match TcpListener::bind(format!("{}:{}", ip, port)) {
            Ok(listener) => {
                Ok(TcpServer { listener, client_counter: 1})
            },
            Err(e) => Err(format!("Error binding to {}:{}. {:?}", ip, port, e))
        }
    }

    pub fn listen(mut self, sender: Sender<bool>) -> Result<thread::JoinHandle<()>, String> {
        match thread::Builder::new().name("listener_thread".to_string()).spawn(move || {
            for stream in self.listener.incoming() {
                match stream {
                    Ok(stream) => {
                        match stream.peer_addr() {
                            Ok(addr) => {
                                log!("New connection from: {}", addr);
                                let game_state = GameState{
                                    client_name: format!("Client{}", self.client_counter).to_string(),
                                    client_score: 0,
                                    server_score: 0,
                                    client_sent_last_ping: false
                                };
                                self.client_counter += 1;
                                Self::handle_incoming(addr, sender.clone(), game_state, stream);
                            },
                            Err(e) => {
                                log!("Error getting peer address: {:?}", e);
                            }
                        }
                    },
                    Err(e) => {
                        log!("Error accepting connection: {:?}", e);
                    }
                }
            }
        }) {
            Ok(join_handle) => Ok(join_handle),
            Err(e) => Err(format!("Error spawning listener thread: {:?}", e))
        }
    }

    /// Should be called after binding to a port.
    pub fn port(&self) -> u16 {
        match self.listener.local_addr() {
            Ok(addr) => addr.port(),
            Err(e) => panic!("Error getting port: {:?}", e)
        }
    }

    fn write_message(stream: &mut TcpStream, buf: String) {
        match stream.write_all(buf.as_bytes()) {
            Ok(_) => {
                stream.flush().unwrap();
                // log!("Sent: {}", buf.trim());
            },
            Err(e) => {
                log!("Error writing to stream: {:?}", e);
            }
        }
    }

    fn handle_incoming(addr: SocketAddr, sender: Sender<bool>, mut game_state: GameState, mut stream: TcpStream) -> () {
        match thread::Builder::new().name(format!("handler_thread_{}", addr)).spawn(move || {
            log!("Handling incoming connection");
            loop {
                let mut buf: String = String::new();
                let mut buf_reader = BufReader::new(&stream);
                match buf_reader.read_line(&mut buf) {
                    Ok(_) => {
                        let buf = buf.trim();
                        if buf == "Ping" {
                            Self::write_message(&mut stream, "Pong\n".to_string());
                            game_state.client_sent_last_ping = true;
                        }
                        else if buf == "Pong" {
                            Self::write_message(&mut stream, "Ping\n".to_string());
                            game_state.client_sent_last_ping = false;
                        } else {
                            if game_state.client_sent_last_ping {
                                game_state.client_score += 1;
                                log!("Client scored! (Client {}: {} - Server: {})", game_state.client_name, game_state.client_score, game_state.server_score);
                            } else {
                                game_state.server_score += 1;
                                log!("Server scored! (Client {}: {} - Server: {})", game_state.client_name, game_state.client_score, game_state.server_score);
                            }
                            let duration = Duration::from_millis(SLEEP_MILLIS_AFTER_SCORING);
                            thread::sleep(duration);
                            if game_state.server_score == SCORE_TO_WIN || game_state.client_score == SCORE_TO_WIN {
                                log!("Game over! Score: (Client {}: {} - Server: {})", game_state.client_name,  game_state.client_score, game_state.server_score);
                                Self::write_message(&mut stream, "GameOver\n".to_string());
                                sender.send(game_state.server_score == SCORE_TO_WIN).unwrap();
                                break;
                            }
                            Self::write_message(&mut stream, "ServeAgain\n".to_string());
                        }
                    },
                    Err(e) => {
                        log!("Error reading from stream: {:?}", e);
                    }
                }
            }
        }) {
            Ok(_) => {},
            Err(e) => {
                panic!("Error spawning thread to handle incoming connection: {:?}", e);
            }
        }
    }

}