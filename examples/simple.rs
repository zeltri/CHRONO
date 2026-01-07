//! Simple terminal emulator example
//!
//! This example demonstrates basic usage of the terminal emulator.
//! Run with: cargo run --example simple

use std::io::Write;
use terminal_ansi::AnsiParser;
use terminal_core::Screen;
use terminal_pty::Pty;

fn main() -> anyhow::Result<()> {
    // Initialize logger
    env_logger::init();

    println!("Simple Terminal Emulator Example");
    println!("=================================\n");

    // Create screen buffer (24 rows x 80 columns)
    let mut screen = Screen::new(24, 80);
    println!(
        "✓ Screen buffer created: {}x{}",
        screen.rows(),
        screen.cols()
    );

    // Create PTY and spawn shell
    let mut pty = Pty::spawn_default_shell(24, 80)?;
    println!("✓ PTY created and shell spawned");

    // Create ANSI parser
    let mut parser = AnsiParser::new();
    println!("✓ ANSI parser initialized\n");

    // Send a simple command
    pty.write(b"echo 'Hello from terminal emulator!'\n")?;
    println!("→ Sent: echo 'Hello from terminal emulator!'");

    // Give shell time to process
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Read output
    let mut buffer = vec![0u8; 4096];
    if let Ok(n) = pty.read(&mut buffer) {
        println!("← Received {} bytes", n);

        // Parse ANSI sequences
        parser.process(&buffer[..n], &mut screen);

        // Display screen content
        println!("\nScreen content:");
        println!("---------------");
        for row in 0..5 {
            // Show first 5 rows
            print!("Row {}: ", row);
            for col in 0..screen.cols() {
                if let Some(cell) = screen.get_cell(row, col) {
                    let ch = cell.get_char();
                    if ch != ' ' && ch != '\0' {
                        print!("{}", ch);
                    } else {
                        print!(" ");
                    }
                }
            }
            println!();
        }
    }

    println!("\n✓ Example completed successfully");
    Ok(())
}
