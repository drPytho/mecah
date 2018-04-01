use database::storage::Storage;
use std::sync::{Arc, Mutex};
use protocol::driver;

use tokio;
use tokio::net::TcpListener;
use tokio::prelude::*;


/// This is the main TCP server structure
///
/// We have the data (which is wrapped in an Arc<Mutex<...>>
/// because we are going to use it in multiple threads at
/// the same time. Also containing host and port to the
/// TcpListener
pub struct Server {
    data: Arc<Mutex<Storage>>,
    host: String,
    port: String,
}

impl Server {
    /// Creates a new server instance with given storage
    ///
    /// # Examples
    ///
    /// ```
    /// use mecah::tcp::server::Server;
    /// use mecah::database::storage::Storage;
    ///
    /// let server = Server::new(Storage::new(), "127.0.0.1".to_string(), "8888".to_string());
    /// ```
    pub fn new(storage: Storage, host: String, port: String) -> Server {
        Server {
            data: Arc::new(Mutex::new(storage)),
            host: host,
            port: port
        }
    }

    /// Run server
    ///
    /// This will block the main thread preventing any more
    /// action from taking place
    ///
    pub fn run(self) -> bool {
        // We 'move' everything from outside the closure
        // inside of it, in this case, 'storage'
        let addr = format!("{}:{}", self.host, self.port).parse().unwrap();
        let listener = TcpListener::bind(&addr).unwrap();

        let server = listener.incoming().for_each(move |socket| {
            println!("Accepted socket; addr={:?}", socket.peer_addr().unwrap());
            let data = self.data.clone();
            driver::handle_conn(socket, data);
            Ok(())
        }).map_err(|err| {
            println!("Accept error = {:?}", err);
        });


        println!("Starting server on {:?}", addr);
        tokio::run(server);
        true
    }
}
