#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]

// External code
use clap::{crate_version, load_yaml, value_t};
use std::net::SocketAddr;

// Internal code
pub mod client;
use client::Client;

pub mod server;
use server::Server;

fn main() {
    let yaml = load_yaml!("main.yml");
    let matches = clap::App::from_yaml(yaml)
        .version(crate_version!())
        .get_matches();

    match matches.subcommand() {
        ("server", Some(matches)) => {
            let bind = value_t!(matches, "bind", SocketAddr).unwrap();
            println!("bind: {:?}", bind);
            let mut server = Server::new(bind);
            server.init();
            server.start();
        }
        ("client", Some(matches)) => {
            let bind = value_t!(matches, "bind", SocketAddr).unwrap();
            let address = value_t!(matches, "address", SocketAddr).unwrap();
            println!("bind: {:?}", bind);
            println!("address: {:?}", address);
            let mut client = Client::new(bind);
            client.init();
            client.connect(address);
        }
        _ => unreachable!(),
    }
}
