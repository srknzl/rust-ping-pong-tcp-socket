use std::process;
use std::thread;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;

mod tcp_server;
mod tcp_client;

macro_rules! log {
    () => {
        println!();
    };
    ($($arg:tt)*) => {{
        println!("Main: {}", format!($($arg)*));
    }};
}

const MISS_CHANCE: f64 = 0.05;
const SCORE_TO_WIN: u8 = 10;
const CLIENT_COUNT: u8 = 15;
const SLEEP_MILLIS_AFTER_SCORING: u64 = 500; 

/// A TCP server-client application as a ping pong game. You can configure number of clients via 
/// CLIENT_COUNT. All games will run concurrently.
/// 
/// The client can send one of "Ping", "Pong" or "Miss" in a loop to the server.
/// Client sends these messages after reading a line from the server. Client also
/// sends of these messages at the start without waiting for the server's response so that the
/// communication can start.
/// 
/// MISS_CHANCE is the chance of sending a "Miss" message between 0.0 and 1.0.
/// 
/// With MISS_CHANCE*100 percent change, a Miss is sent by the client.
/// With (1-MISS_CHANCE)*100/2 percent change, a Ping is sent by the client.
/// With (1-MISS_CHANCE)*100/2 percent change, a Pong is sent by the client.
/// 
/// Server sends Ping to a Pong message, and a Pong to a Ping message. 
/// If a Miss is sent by the client, server increments the score of
/// the entity who sent the last Ping.
/// 
/// Server waits SLEEP_MILLIS_AFTER_SCORING milliseconds after the client or the server scores.
/// 
/// When score of one entity reaches SCORE_TO_WIN, the game ends,
/// and the server sends a "GameOver" message, which will make client stop.
fn main() {
    let port: u16;
    let server_join_handle: thread::JoinHandle<()>;
    let (tx, rx): (Sender<bool>, Receiver<bool>) = mpsc::channel();

    match tcp_server::TcpServer::new("127.0.0.1", 0) {
        Ok(server) => {
            port = server.port();
            match server.listen(tx) {
                Ok(join_handle) => {
                    server_join_handle = join_handle;
                    log!("Server started listening on port {}", port);
                },
                Err(e) => {
                    log!("Server could not start listening on port {}: {}", port, e);
                    process::exit(1);
                }
            }
        },
        Err(e) => {
            log!("Error starting server: {}", e);
            process::exit(1);
        }
    }

    for _ in 0..CLIENT_COUNT  {
        match tcp_client::TcpClient::new("127.0.0.1", MISS_CHANCE, port) {
            Ok(client) => {
                match client.run() {
                    Ok(_) => {
                        log!("Client started");
                    },
                    Err(e) => {
                        log!("Client could not start: {}", e);
                        process::exit(1);
                    }
                }
            },
            Err(e) => {
                log!("Error starting client: {}", e);
                process::exit(1)
            }
        }
    }
    
    let mut server_won_count = 0;

    for _ in 0..CLIENT_COUNT {
        let server_won = rx.recv().unwrap();
        if server_won {
            server_won_count += 1;
        }
    }

    log!("Server won {} times", server_won_count);

    match server_join_handle.join() {
        Ok(_) => {
            log!("Server finished");
        },
        Err(e) => {
            log!("Error joining server thread: {:?}", e);
        }
    }
}
