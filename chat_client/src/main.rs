#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_assignments)]

use std::io::{self, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;

const LOCAL: &str = "10.0.0.87:6000";
const MSG_SIZE: usize = 64;

fn main() {
    let mut client = TcpStream::connect(LOCAL).expect("Stream failed to connect");
    client
        .set_nonblocking(true)
        .expect("Failed to Initialize Nonblocking");

    let (tx, rx) = mpsc::channel::<String>();

    thread::spawn(move || loop {
        let mut buff = vec![0; MSG_SIZE];
        match client.read_exact(&mut buff) {
            Ok(_) => {
                let msg = buff
                    .into_iter()
                    .take_while(|&x| x != 0_)
                    .collect::<Vec<_>>();
                println!("Message Recv {:?}", msg);
            }
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            Err(_) => {
                println!("Connection with server was severed");
                break;
            }
        }
        match rx.try_recv() {
            Ok(msg) => {
                let mut buff = msg.clone().into_bytes();
                buff.resize(MSG_SIZE, 0);
                client.write_all(&buff).expect("Writing to Socket Failed");
                println!("Message sent {:?}", msg);
            }
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => break,
        }

        thread::sleep(Duration::from_millis(100));
    });
    println!("Write a Message:");
    loop {
        let mut buff = String::new();
        io::stdin()
            .read_line(&mut buff)
            .expect("Reading from stdin Failed!");
        let msg = buff.trim().to_string();
        if msg == ":quit" || tx.send(msg).is_err() {
            break;
        };
    }
    println!("Peace out!");
}
