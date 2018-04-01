use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use tokio;
use tokio::io;
use tokio::net::TcpStream;
use tokio::prelude::*;

use bincode::{deserialize_from, serialize};

use bytes::{BytesMut, IntoBuf, Buf, BufMut};

use futures::Future;

use database::storage::{Storage, StorageError};
use protocol::protocol::{Message, Request, Response, OpCode};

struct MessageStream {
    socket: TcpStream,
    rd: BytesMut,
    wr: BytesMut,
}

impl MessageStream {
    /// Create a new `MessageStream` codec backed by the socket
    fn new(socket: TcpStream) -> Self {
        MessageStream {
            socket,
            rd: BytesMut::new(),
            wr: BytesMut::new(),
        }
    }

    fn poll_read_to_buf(&mut self) -> Result<Async<()>, io::Error> {
        loop {
            // Ensure the read buffer has capacity.
            //
            // This might result in an internal allocation.
            self.rd.reserve(1024);

            // Read data into the buffer.
            //
            // The `read_buf` fn is provided by `AsyncRead`.
            let n = try_ready!(self.socket.read_buf(&mut self.rd));

            if n == 0 {
                return Ok(Async::Ready(()));
            }
        }
    }

    fn write_response(&mut self, res: Response) {
        // Push the line onto the end of the write buffer.
        println!("Before {:?}", self.wr);
        self.wr.put(serialize(&Message::Res(res)).unwrap());
        println!("After {:?}", self.wr);
    }

    fn poll_flush_buf(&mut self) -> Poll<(), io::Error> {
        // As long as there is buffered data to write, try to write it.

        println!("Going to send {:?} bytes", self.wr.len());
        while !self.wr.is_empty() {
            println!("Sending...");
            // Try to read some bytes from the socket
            let n = try_ready!(self.socket.poll_write(&self.wr));

            println!("Sent {:?} bytes.", n);
            // As long as the wr is not empty, a successful write should
            // never write 0 bytes.
            assert!(n > 0);

            // This discards the first `n` bytes of the buffer.
            self.wr.advance(n);
        }

        Ok(Async::Ready(()))
    }
}

impl Stream for MessageStream {
    type Item = Message;
    type Error = ();

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        // Fill up the buffer with all available data.
        let sock_closed = self.poll_read_to_buf().expect("Should not be an error").is_ready();

        let mut rdr = self.rd.clone().into_buf().reader();
        return match deserialize_from(&mut rdr) {
            Ok(m) => {
                self.rd.advance(rdr.get_ref().position() as usize); 
                Ok(Async::Ready(Some(m)))
            },
            Err(_) => {
                if sock_closed {
                    Ok(Async::Ready(None))
                } else {
                    Ok(Async::NotReady)
                }
            }
        }
    }
}

struct Client {
    ms: MessageStream,
    data: Arc<Mutex<Storage>>,
    addr: SocketAddr,
}

impl Client {
    fn new(ms: MessageStream, data: Arc<Mutex<Storage>>) -> Client {
        // Get the client socket address
        let addr = ms.socket.peer_addr().unwrap();

        Client {
            ms,
            data,
            addr
        }
    }
}

impl Future for Client {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        // Read new lines from the socket
        while let Async::Ready(dat) = self.ms.poll()? {
            println!("Received Message {:?} from {:?}", dat, self.addr);

            if let Some(message) = dat {
                let res = match message {
                    Message::Req(r) => do_request(r, self.data.clone()),
                    Message::Res(_) => Response::Failed("I am the one who respondes".to_string()),
                };
                println!("Responding with {:?}", res);
                self.ms.write_response(res);
                match self.ms.poll_flush_buf() {
                    Err(_) => return Err(()),
                    _ => {},
                }
                
            } else {
                // EOF was reached. The remote client has disconnected.
                // There is nothing more to do.
                return Ok(Async::Ready(()));
            }
        }

        // As always, it is important to not just return `NotReady`
        // without ensuring an inner future also returned `NotReady`.
        Ok(Async::NotReady)
    }
}

pub fn handle_conn(socket: TcpStream, data: Arc<Mutex<Storage>>) {
    let ms = MessageStream::new(socket);
    let client = Client::new(ms, data);

    tokio::spawn(client);
}

pub fn do_request(req: Request, data: Arc<Mutex<Storage>>) -> Response {
    println!("Processing {:?}", req);
    let mut data = data.lock().unwrap();

    match req.op {
        OpCode::Set => {
            let key = req.key;
            let val = req.val;
            match data.set(key, val) {
                Ok(_) => Response::Success,
                Err(_) => Response::Failed("Internal Error".to_string())
            }
        },
        OpCode::Get => {
            let key = req.key;
            match data.get(key) {
                Ok(val) => Response::Return(val.to_string()),
                Err(StorageError::KeyNotFound) => Response::Failed("Key not found".to_string()),
                Err(_) => Response::Failed("Internal Error".to_string()),
            }
        },
        OpCode::Del => {
            let key = req.key;
            match data.del(key) {
                Ok(_) => Response::Success,
                Err(StorageError::KeyNotFound) => Response::Failed("Key not found".to_string()),
                Err(_) => Response::Failed("Internal Error".to_string())
            }

        }
    }
}
