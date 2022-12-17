use std::process;
use std::thread;

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

fn main() {
    let port: u16;
    let client_join_handle: thread::JoinHandle<()>;
    let server_join_handle: thread::JoinHandle<()>;

    match tcp_server::TcpServer::new("127.0.0.1", 0) {
        Ok(server) => {
            port = server.port();
            match server.listen() {
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

    match tcp_client::TcpClient::new("127.0.0.1", port) {
        Ok(client) => {
            match client.run() {
                Ok(join_handle) => {
                    client_join_handle = join_handle;
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

    
    match client_join_handle.join() {
        Ok(_) => {
            log!("Client finished");
        },
        Err(e) => {
            log!("Error joining client thread: {:?}", e);
        }
    }

    match server_join_handle.join() {
        Ok(_) => {
            log!("Server finished");
        },
        Err(e) => {
            log!("Error joining server thread: {:?}", e);
        }
    }
}
