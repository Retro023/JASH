// Banner printing

//imports
use colored::Colorize;
use std::io::{self, Write};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::{thread, time::Duration};

// banner tells user who JASH connected to
pub fn connected_banner(ip: &str, port: &str) {
    println!(); // newline
    let barrier = ip.chars().count() + 19 + port.chars().count(); // barrier is length of the string, ip and port 
    println!("{}", "=".repeat(barrier).bright_cyan().bold());
    println!("🔗 Connection to: {}:{}", ip, port);
    println!("{}", "=".repeat(barrier).magenta().bold());
}

// prompt to tell user JASH is Listing on port

pub fn listener_banner(port: &str) {
    let port_owned = port.to_owned();
    let port_for_main = port_owned.clone();
    let done = Arc::new(AtomicBool::new(false));
    let done_clone = Arc::clone(&done);

    // spinner
    let spinner_handle = thread::spawn(move || {
        let spinner_frames = [
            "|".bright_blue().bold(),
            "/".bright_magenta().bold(),
            "\\".bright_purple().bold(),
        ];
        let mut idx = 0;

        while !done_clone.load(Ordering::SeqCst) {
            print!(
                "\r{} {} {}",
                "Starting Listener".bright_yellow(),
                port_owned,
                spinner_frames[idx % spinner_frames.len()]
            );
            io::stdout().flush().unwrap();
            thread::sleep(Duration::from_millis(150));
            idx += 1;
        }
    });

    // Simulate listener setup (replace this with actual logic)
    thread::sleep(Duration::from_secs(3));

    done.store(true, Ordering::SeqCst);
    spinner_handle.join().unwrap();

    // variable out of place on purpose prints "-" under and across output
    println!(
        "\r{} {} {} 
{}",
        "✓".bright_green().bold(),
        "Listener started on".bright_magenta(),
        port_for_main.bright_cyan(),
        "-".repeat(30).bright_yellow()
    );
}

// Copyright (c) 2025 MuteAvery
// Licensed under the MIT License. See LICENSE file in the project root
