use regex::Regex;
use std::env;
use std::io::{self, Write};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::task::JoinHandle;
use tokio::time::{interval, sleep};
use tracing::info;

const TMUX_CMD: &str = "tmux";

async fn send_to_terminal(command: &str) -> io::Result<()> {
    let status = Command::new(TMUX_CMD)
        .args(&["send-keys", command, "Enter"])
        .status()
        .await?;
    if status.success() {
        println!("\x1b[94m[SENT TO TMUX]:\x1b[0m {}", command);
    } else {
        eprintln!("\x1b[91m[ERROR]: tmux exited with {}\x1b[0m", status);
    }
    Ok(())
}

async fn send_keystrokes_without_enter(text: &str) -> io::Result<()> {
    let status = Command::new(TMUX_CMD)
        .args(&["send-keys", text])
        .status()
        .await?;
    if status.success() {
        println!("\x1b[94m[SENT TO TMUX]:\x1b[0m {} (no Enter)", text);
    } else {
        eprintln!("\x1b[91m[ERROR]: tmux exited with {}\x1b[0m", status);
    }
    Ok(())
}

async fn send_enter() -> io::Result<()> {
    let status = Command::new(TMUX_CMD)
        .args(&["send-keys", "Enter"])
        .status()
        .await?;
    if status.success() {
        println!("\x1b[94m[SENT TO TMUX]:\x1b[0m [ENTER]");
    } else {
        eprintln!("\x1b[91m[ERROR]: tmux exited with {}\x1b[0m", status);
    }
    Ok(())
}

async fn send_esc() -> io::Result<()> {
    let status = Command::new(TMUX_CMD)
        .args(&["send-keys", "Escape"])
        .status()
        .await?;
    if status.success() {
        println!("\x1b[94m[SENT TO TMUX]:\x1b[0m [ESC]");
    } else {
        eprintln!("\x1b[91m[ERROR]: tmux exited with {}\x1b[0m", status);
    }
    Ok(())
}
async fn send_ctrl_c() -> io::Result<()> {
    let status = Command::new(TMUX_CMD)
        .args(&["send-keys", "C-c"])
        .status()
        .await?;
    if status.success() {
        println!("\x1b[94m[SENT TO TMUX]:\x1b[0m [CTRL-C]");
    } else {
        eprintln!("\x1b[91m[ERROR]: tmux exited with {}\x1b[0m", status);
    }
    Ok(())
}
async fn wait_for_exit_command() {
    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin).lines();
    let mut ticker = interval(Duration::from_secs(3));

    loop {
        tokio::select! {
            _ = ticker.tick() => {
                // ⏎ Always send enter every tick
                if let Err(e) = send_enter().await {
                    eprintln!("Error sending enter: {:?}", e);
                } else {
                    info!("Tick: sent enter");
                }
            }

            line = reader.next_line() => {
                match line {
                    Ok(Some(input)) => {
                        if input.trim() == "/" {
                            info!("Received /, exiting...");
                            break;
                        }
                    }
                    Ok(None) => break, // EOF
                    Err(e) => {
                        eprintln!("Error reading line: {:?}", e);
                        break;
                    }
                }
            }
        }
    }
}

pub async fn test_send_to_terminal(prompt: &str) -> io::Result<()> {
    println!("\nStep 1: Clearing and starting Gemini");
    tokio::time::sleep(Duration::from_secs(2)).await;

    println!("\nStep 2.5: Pressing ESC to clear any previous input");
    send_esc().await?;

    println!("\nStep 3: Type prompt");
    send_keystrokes_without_enter(prompt).await?;
    tokio::time::sleep(Duration::from_secs(1)).await;
    send_enter().await?;
    println!("\nStep 4: Type `/` at any time to cancel...");
    wait_for_exit_command().await;
    info!("Exiting terminal task");
    Ok(())
}
