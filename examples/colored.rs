//! Colored output terminal example
//!
//! This example demonstrates ANSI color support and context detection.
//! Run with: cargo run --example colored

use std::io::Write;
use terminal_ansi::AnsiParser;
use terminal_core::{analyze_line_context, Cell, LineContext, Screen};
use terminal_pty::Pty;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    println!("Colored Terminal Emulator Example");
    println!("==================================\n");

    // Create components
    let mut screen = Screen::new(30, 100);
    let mut pty = Pty::spawn_default_shell(30, 100)?;
    let mut parser = AnsiParser::new();

    println!("Testing ANSI color codes...\n");

    // Test 1: Basic colors
    let test_commands = vec![
        "echo -e '\\033[31mRed text\\033[0m'",
        "echo -e '\\033[32mGreen text\\033[0m'",
        "echo -e '\\033[33mYellow text\\033[0m'",
        "echo -e '\\033[34mBlue text\\033[0m'",
        "echo -e '\\033[1;35mBold Magenta\\033[0m'",
        "echo -e '\\033[36mCyan text\\033[0m'",
    ];

    for cmd in &test_commands {
        pty.write(cmd.as_bytes())?;
        pty.write(b"\n")?;
        println!("→ {}", cmd);

        // Wait and read
        std::thread::sleep(std::time::Duration::from_millis(50));

        let mut buffer = vec![0u8; 4096];
        if let Ok(n) = pty.read(&mut buffer) {
            parser.process(&buffer[..n], &mut screen);
        }
    }

    // Test 2: File listing colors
    println!("\nTesting file listing detection...");
    pty.write(b"ls -la\n")?;
    std::thread::sleep(std::time::Duration::from_millis(100));

    let mut buffer = vec![0u8; 8192];
    if let Ok(n) = pty.read(&mut buffer) {
        parser.process(&buffer[..n], &mut screen);

        // Analyze some lines for context
        println!("\nAnalyzing line contexts:");
        for row in 0..10 {
            let mut line = String::new();
            for col in 0..screen.cols() {
                if let Some(cell) = screen.get_cell(row, col) {
                    let ch = cell.get_char();
                    if ch != '\0' {
                        line.push(ch);
                    }
                }
            }

            if !line.trim().is_empty() {
                let context = analyze_line_context(&line);
                match context {
                    LineContext::FileList(_) => println!("  Row {}: FileList detected", row),
                    LineContext::Error => println!("  Row {}: Error detected", row),
                    LineContext::Warning => println!("  Row {}: Warning detected", row),
                    LineContext::Normal => {}
                    _ => {}
                }
            }
        }
    }

    // Test 3: Error simulation
    println!("\nTesting error detection...");
    pty.write(b"echo 'Error: Something went wrong'\n")?;
    pty.write(b"echo 'Warning: This is a warning'\n")?;

    std::thread::sleep(std::time::Duration::from_millis(100));

    if let Ok(n) = pty.read(&mut buffer) {
        parser.process(&buffer[..n], &mut screen);
    }

    println!("\n✓ All color tests completed");
    println!("\nScreen statistics:");
    println!("  Rows: {}", screen.rows());
    println!("  Cols: {}", screen.cols());
    println!(
        "  Cursor: ({}, {})",
        screen.cursor().row,
        screen.cursor().col
    );

    Ok(())
}
