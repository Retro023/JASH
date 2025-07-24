// clean IDE
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]
// imports

mod shell;
mod ui;
use clap::{Arg, Command};
use std::env;
use std::net::{TcpListener, TcpStream};
use std::process;
use std::thread;

fn main() {
    let matches = Command::new("JASH")
        .version("0.1.2")
        .author("MuteAvery")
        .about("Just Another Shell Handler")
        .arg(
            Arg::new("port")
                .short('p')
                .long("port")
                .help("the port to listen on")
                .required(false),
        )
        .get_matches();

    let port = matches.get_one::<String>("port").unwrap();

    let bind_addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&bind_addr).expect("Failed to bind to port");
    ui::listener_banner(&port);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let peer = stream
                    .peer_addr()
                    .map(|addr| addr.ip().to_string())
                    .unwrap_or_else(|_| "unknown".to_string());

                ui::connected_banner(&peer, &port);

                let stream_clone = stream.try_clone().expect("Failed to clone stream");
                shell::stabilise_shell(&stream_clone);

                shell::handle_client(stream, peer);
            }
            Err(e) => {
                eprintln!("Falied to connect: {}", e);
            }
        }
    }
}

// Copyright (c) 2025 MuteAvery
// Licensed under the MIT License. See LICENSE file in the project root
