#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]

// External code
use bincode::{deserialize, serialize};
use crossbeam_channel::{Receiver, RecvError, SendError, Sender};
use laminar::{ErrorKind, Packet, Socket, SocketEvent};
use serde_derive::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::thread;

// Internal code
use crate::client;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Message {
    Pong,
}

pub struct Server {
    packet_tx: Sender<Packet>,
    event_rx: Receiver<SocketEvent>,
    init: bool,
    socket: Option<Socket>,
}

enum ClientStatus {
    Established,
    Connected,
    Disconnected,
}

// Server representation of client
struct ClientRepr {
    pub status: ClientStatus,
    pub ping_count: u64,
    pub ping_rtt_sum: u64,
    pub ping_rtt_avg: u16,
}

impl Server {
    pub fn new(bind: SocketAddr) -> Self {
        let (mut socket, packet_tx, event_rx) = Socket::bind(bind).unwrap();
        Self {
            packet_tx,
            event_rx,
            init: false,
            socket: Some(socket),
        }
    }

    pub fn init(&mut self) -> &Self {
        if !self.init {
            let mut socket = self.socket.take().unwrap();
            thread::spawn(move || socket.start_polling());
            self.init = true;
        }
        self
    }

    pub fn start(&mut self) {
        if !self.init {
            eprintln!("Server not initialized!");
            return;
        }
        loop {
            println!("receiving...");
            if let Ok(Some(packet)) = self.recv() {
                let address = packet.addr();
                if let Some(message) = deserialize(packet.payload()).ok() {
                    self.action(address, message);
                }
            }
        }
    }

    /// Send message to client at address
    pub fn send(&mut self, address: SocketAddr, message: Message) -> Result<(), SendError<Packet>> {
        match serialize(&message) {
            Ok(data) => {
                let packet = Packet::reliable_unordered(address, data);
                self.packet_tx.send(packet)?;
            }
            Err(_) => { /* Failed to serialize. TODO: Logging */ }
        }
        Ok(())
    }

    /// Receive message packet
    pub fn recv(&mut self) -> Result<Option<Packet>, RecvError> {
        match self.event_rx.recv()? {
            SocketEvent::Packet(packet) => {
                return Ok(Some(packet));
            }
            SocketEvent::Timeout(address) => { /* TODO: Logging */ }
            SocketEvent::Connect(address) => { /* TODO: Logging */ }
        }
        Ok(None)
    }

    pub fn action(&mut self, address: SocketAddr, message: client::Message) {
        match message {
            client::Message::Ping => {
                if self.pong(address) {
                    println!("ping {}", address);
                }
            }
        }
    }

    pub fn pong(&mut self, address: SocketAddr) -> bool {
        self.send(address, Message::Pong).is_ok()
    }
}
