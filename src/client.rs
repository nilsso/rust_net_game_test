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
use std::time::Duration;

// Internal code
use crate::server;
use server::Message as ServerMessage;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Message {
    Ping,
}

pub struct Client {
    init: bool,
    socket: Option<Socket>,
    packet_tx: Sender<Packet>,
    event_rx: Receiver<SocketEvent>,
    server: Option<ServerRepr>,
}

// Client representation of server
struct ServerRepr {
    pub address: SocketAddr,
}

impl Client {
    pub fn new(bind: SocketAddr) -> Self {
        let (mut socket, packet_tx, event_rx) = Socket::bind(bind).unwrap();
        Self {
            init: false,
            socket: Some(socket),
            packet_tx,
            event_rx,
            server: None,
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

    pub fn connect(&mut self, address: SocketAddr) {
        if !self.init {
            eprintln!("Client not initialized!");
            return;
        }
        self.server = Some(ServerRepr { address });
        loop {
            if self.ping() {
                println!("pong {}", address);
                thread::sleep(Duration::from_millis(500));
            }
        }
    }

    /// Send message to server
    pub fn send(&mut self, message: Message) -> Result<(), SendError<Packet>> {
        if let Some(server) = &self.server {
            match serialize(&message) {
                Ok(data) => {
                    let packet = Packet::reliable_unordered(server.address, data);
                    self.packet_tx.send(packet)?;
                }
                Err(e) => {
                    eprintln!("error {}", e);
                }
            }
        }
        Ok(())
    }

    /// Receive message packet
    pub fn recv(&mut self) -> Result<Option<Packet>, RecvError> {
        match self.event_rx.recv()? {
            SocketEvent::Packet(packet) => {
                return Ok(Some(packet));
            }
            SocketEvent::Timeout(address) => {
                println!("{} timed out", address);
            }
            SocketEvent::Connect(address) => {
                println!("{} connection", address);
            }
        }
        Ok(None)
    }

    pub fn ping(&mut self) -> bool {
        if self.send(Message::Ping).is_ok() {
            if let Ok(Some(packet)) = self.recv() {
                let data = deserialize::<ServerMessage>(packet.payload());
                if data.is_ok() && data.ok() == Some(ServerMessage::Pong) {
                    return true;
                }
            }
        }
        false
    }
}
