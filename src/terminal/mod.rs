#!/usr/bin/env rust-script
use std::io::{self};
use std::process::Command;
use std::{thread, time::Duration};

const TMUX_CMD: &str = "tmux";

fn send_to_terminal(command: &str) -> io::Result<()> {
    let status = Command::new(TMUX_CMD)
        .args(&["send-keys", command, "Enter"])
        .status()?;
    if status.success() {
        println!("\x1b[94m[SENT TO TMUX]:\x1b[0m {}", command);
    } else {
        eprintln!("\x1b[91m[ERROR]: tmux exited with {}\x1b[0m", status);
    }
    Ok(())
}

fn send_keystrokes_without_enter(text: &str) -> io::Result<()> {
    let status = Command::new(TMUX_CMD).args(&["send-keys", text]).status()?;
    if status.success() {
        println!("\x1b[94m[SENT TO TMUX]:\x1b[0m {} (no Enter)", text);
    } else {
        eprintln!("\x1b[91m[ERROR]: tmux exited with {}\x1b[0m", status);
    }
    Ok(())
}

fn send_enter() -> io::Result<()> {
    let status = Command::new(TMUX_CMD)
        .args(&["send-keys", "Enter"])
        .status()?;
    if status.success() {
        println!("\x1b[94m[SENT TO TMUX]:\x1b[0m [ENTER]");
    } else {
        eprintln!("\x1b[91m[ERROR]: tmux exited with {}\x1b[0m", status);
    }
    Ok(())
}

fn send_esc() -> io::Result<()> {
    let status = Command::new(TMUX_CMD)
        .args(&["send-keys", "Escape"])
        .status()?;
    if status.success() {
        println!("\x1b[94m[SENT TO TMUX]:\x1b[0m [ESC]");
    } else {
        eprintln!("\x1b[91m[ERROR]: tmux exited with {}\x1b[0m", status);
    }
    Ok(())
}

pub fn test_send_to_terminal(prompt: &str) -> io::Result<()> {
    println!("\nStep 2.5: Pressing ESC to clear any previous input");
    send_esc()?;
    thread::sleep(Duration::from_secs(1));

    println!("\nStep 3: Type prompt");
    send_keystrokes_without_enter(prompt)?;
    thread::sleep(Duration::from_secs(1));

    println!("\nStep 4: Press Enter");
    send_enter()?;

    println!("\nStep 5: Waiting for Gemini response...");
    thread::sleep(Duration::from_secs(60));

    println!("\nStep 6: Auto-accepting file creation...");
    send_enter()?;

    Ok(())
}

// pub fn terminal_test() {
//     if let Err(e) = test_send_to_terminal() {
//         eprintln!("\x1b[91m[ERROR]: {}\x1b[0m", e);
//     }
// }
