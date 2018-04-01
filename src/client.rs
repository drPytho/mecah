extern crate bincode;
extern crate bytes;
extern crate mecah;

use std::io;
use std::io::BufReader;
use std::io::Write;
use std::net::TcpStream;

use bincode::deserialize_from;
use mecah::protocol::protocol::{mangle, Message, OpCode, Request};
use mecah::*;

fn main() {
    let mut host: String = "".to_string();
    let mut port: String = "".to_string();

    util::args::parse_host_args(&mut host, &mut port, "Mecah command line interface");

    cli(&host, &port);
}

fn cli(host: &String, port: &String) {
    let server = format!("{}:{}", host, port);
    let stream = TcpStream::connect(&*server);

    match stream {
        Err(e) => println!("Unable to connect: {}", e),
        Ok(mut stream) => loop {
            cli_loop(host, port, &mut stream);
        },
    }
}

fn cli_loop(host: &String, port: &String, stream: &mut TcpStream) {
    print!("{}:{}> ", host, port);
    io::stdout().flush().unwrap();

    let input = util::io::read_line().trim().to_string();

    if input == "" {
        println!("");
        return;
    }

    let message = parse_command(&input);
    if message.is_err() {
        println!("{}, is an invalid command.", input);
        return;
    }
    let message = message.unwrap();

    println!("Message sending to server: {:?}", message);

    let dat = mangle(&message);
    stream.write(&dat).unwrap();
    let reader = BufReader::new(stream);

    let msg: Message = match deserialize_from(reader) {
        Ok(m) => m,
        Err(_) => {
            println!("Could not read responce");
            return;
        }
    };
    println!("Responce: {:?}", msg);
}

fn parse_command(command: &String) -> Result<Message, ()> {
    let words: Vec<&str> = command.split(" ").collect();

    if words.len() < 1 {
        return Err(());
    }

    let req = match words[0] {
        "get" => {
            if words.len() < 2 {
                return Err(());
            }
            Request {
                op: OpCode::Get,
                key: words[1].to_string(),
                val: "".to_string(),
            }
        }
        "set" => {
            if words.len() < 3 {
                return Err(());
            }
            Request {
                op: OpCode::Set,
                key: words[1].to_string(),
                val: words[2].to_string(),
            }
        }
        "del" => {
            if words.len() < 2 {
                return Err(());
            }
            Request {
                op: OpCode::Del,
                key: words[1].to_string(),
                val: "".to_string(),
            }
        }
        _ => return Err(()),
    };

    Ok(Message::Req(req))
}
