use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream, SocketAddr};
use std::thread;

pub struct TcpServer {
    listener: TcpListener
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
                Ok(TcpServer { listener })
            },
            Err(e) => Err(format!("Error binding to {}:{}. {:?}", ip, port, e))
        }
    }

    pub fn listen(self) -> Result<thread::JoinHandle<()>, String> {
        match thread::Builder::new().name("listener_thread".to_string()).spawn(move || {
            for stream in self.listener.incoming() {
                match stream {
                    Ok(stream) => {
                        match stream.peer_addr() {
                            Ok(addr) => {
                                log!("New connection from: {}", addr);
                                self.handle_incoming(addr, stream);
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

    fn handle_incoming(&self, addr: SocketAddr, mut stream: TcpStream) -> () {
        match thread::Builder::new().name(format!("handler_thread_{}", addr)).spawn(move || {
            log!("Handling incoming connection");
        }) {
            Ok(_) => {
                loop {
                    let mut buf: String = String::new();
                    match stream.read_to_string(&mut buf) {
                        Ok(_) => {
                            match stream.write_all(buf.as_bytes()) {
                                Ok(_) => {
                                    stream.flush().unwrap();
                                    log!("Echoed back to client: {}", buf);
                                },
                                Err(e) => {
                                    log!("Error writing to stream: {:?}", e);
                                }
                            }
                        },
                        Err(e) => {
                            log!("Error reading from stream: {:?}", e);
                        }
                    }
                }
            },
            Err(e) => {
                log!("Error spawning thread to handle incoming connection: {:?}", e);
            }
        }
    }

}