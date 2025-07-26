// clean IDE
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]
// imports

mod help;
mod shell;
mod ui;
use clap::builder::TypedValueParser;
use clap::{Arg, ArgAction, Command};
use colored::Colorize;
use ctrlc::set_handler;
use std::env;
use std::net::{TcpListener, TcpStream};
use std::process;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread;
//current version of JASH
const JASH_VERSION: &str = "JASHv 0.1.2.1 Fortunate";

fn main() {
    let matches = Command::new("JASH")
        .version(JASH_VERSION)
        .author("MuteAvery")
        .about("Just Another Shell Handler")
        .disable_help_flag(true)
        .disable_version_flag(true)
        .arg(
            Arg::new("port")
                .short('p')
                .long("port")
                .help("the port to listen on")
                .required(false),
        )
        .arg(
            Arg::new("help")
                .short('h')
                .long("help")
                .help("Help menu")
                .action(ArgAction::SetTrue)
                .required(false),
        )
        .arg(
            Arg::new("Tldr_help")
                .short('t')
                .long("helptldr")
                .action(ArgAction::SetTrue)
                .required(false),
        )
        .arg(
            Arg::new("version")
                .short('v')
                .long("version")
                .action(ArgAction::SetTrue)
                .required(false),
        )
        .get_matches();

    // print the large help output
    if matches.get_flag("help") {
        help::large_help_output();
        process::exit(0);
    }
    // print the tldr help output
    if matches.get_flag("Tldr_help") {
        help::tldr_help();
        process::exit(0)
    }
    if matches.get_flag("version") {
        println!("{}", JASH_VERSION);
        process::exit(0)
    }

    let port = matches
        .get_one::<String>("port")
        .map(|s| s.as_str())
        .unwrap_or("1337");

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        println!(
            "{}",
            "\n [!!] Ctrl+c Detected shutting down JASH listener...".red()
        );
        std::process::exit(0);
    })
    .expect("Error setting ctrlc handler");

    let bind_addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&bind_addr).expect("Failed to bind to port");
    listener
        .set_nonblocking(false)
        .expect("Cannot set non-blocking");
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
