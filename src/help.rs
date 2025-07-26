use colored::Colorize;
// complete help output
pub fn large_help_output() {
    let version = "JASHv 0.1.2.A Fortunate".bright_magenta().bold();
    println!("{}\n", version);
    println!("{}", "Usage:".cyan().bold());
    println!("JASH [Flags] [Options]\n");

    println!("{}", "Options".cyan().bold());
    println!("  -p  --port <port>       set the port to listen on");
    println!("  -v --version            print the current version of JASH");
    println!("  -h  --help              print this help menu");
    println!(" -htldr  --help-tldr      print a short tldr version of this help menu\n");

    println!("{}", "Examples".cyan().bold());
    println!("JASH -p 1337");
    println!("JASH --port 4444\n");

    println!("{}", "Features".cyan().bold());
    println!("  ✅ TTY-stabilized reverse shell");
    println!("  ✅ Clean UI with listener and connection banners");
    println!("  ✅ ctrl+q to exit shell");
    println!("  ✅ Auto PTY setup and shell resizing\n");

    println!("{}", "Want to contribute?".bright_green().bold());
    println!("{}", "https://github.com/Retro023/JASH\n".blue().bold());

    println!("{}", "Want to support me?".bright_yellow().bold());
    println!("{}", "https://buymeacoffee.com/muteavery\n".blue().bold());

    println!("License MIT");
}

// tldr version of the help menu
pub fn tldr_help() {
    println!("Usage:\n JASH [FLAGS] [OPTIONS]\n");
    println!("Try JASH --help for more info");
}
