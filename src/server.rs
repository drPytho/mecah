extern crate argparse;
extern crate mecah;

use mecah::database::storage::Storage;
use mecah::tcp::server;

fn main() {
    let mut host: String = "".to_string();
    let mut port: String = "".to_string();

    mecah::util::args::parse_host_args(&mut host, &mut port, "Mecah server");

    println!("Starting the Mecah server");
    println!("Listening on {}:{}", host, port);

    // We need to wrap our storage in a Arc with Mutex:
    // Arc in order to have atomic reference counting and
    // Mutex to prevent data races between threads
    let storage = Storage::new();
    let server = server::Server::new(storage, host, port);

    server.run();
}
