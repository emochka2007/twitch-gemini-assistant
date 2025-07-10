use regex::Regex;
use std::env;
use std::io::{self, Write};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::task::JoinHandle;
use tokio::time::{interval, sleep};
use tracing::info;

pub const TMUX_CMD: &str = "tmux";

pub async fn send_to_terminal(command: &str) -> io::Result<()> {
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

pub async fn send_vscode_enable_custom_css() -> io::Result<()> {
    // Step 1: Simulate Cmd+Shift+P to open the Command Palette
    // In tmux, "C-S-p" often maps to Ctrl+Shift+P, which works in most cases
    let open_palette = Command::new(TMUX_CMD)
        .args(&["send-keys", "C-S-p"])
        .status()
        .await?;

    if !open_palette.success() {
        eprintln!("[ERROR] Failed to send Cmd+Shift+P");
        return Ok(());
    }

    // Wait a moment to ensure the palette opens
    sleep(Duration::from_millis(300)).await;

    // Step 2: Type the command
    let command = "Enable Custom CSS and JS";

    let type_command = Command::new(TMUX_CMD)
        .args(&["send-keys", command])
        .status()
        .await?;

    if !type_command.success() {
        eprintln!("[ERROR] Failed to type VSCode command");
        return Ok(());
    }

    // Step 3: Send Enter
    let press_enter = Command::new(TMUX_CMD)
        .args(&["send-keys", "Enter"])
        .status()
        .await?;

    if !press_enter.success() {
        eprintln!("[ERROR] Failed to press Enter");
        return Ok(());
    }

    println!("\x1b[92m✅ Sent: Cmd+Shift+P → {command} → Enter\x1b[0m");
    Ok(())
}

pub async fn send_shortcut_to_vscode() -> std::io::Result<()> {
    let applescript = r#"
        tell application "Visual Studio Code"
            activate
        end tell
        delay 0.3
        tell application "System Events"
            -- Send Cmd+Option+U
            keystroke "u" using {command down, option down}
            delay 0.2
            -- Send Cmd+Option+R
            keystroke "r" using {command down, option down}
        end tell
    "#;

    let status = Command::new("osascript")
        .arg("-e")
        .arg(applescript)
        .status()
        .await?;

    if status.success() {
        println!("✅ Sent Cmd+Opt+U to VSCode");
    } else {
        eprintln!("❌ Failed to send shortcut");
    }

    Ok(())
}

pub async fn restart_vscode() -> io::Result<()> {
    // Kill all running VSCode instances
    let kill_status = Command::new("pkill")
        .arg("-f")
        .arg("Visual Studio Code")
        .status()
        .await?;

    if kill_status.success() {
        println!("\x1b[93m[VSCode]: Killed running instances\x1b[0m");
    } else {
        println!("\x1b[93m[VSCode]: No running instances or pkill failed\x1b[0m");
    }

    // Wait briefly to ensure it's fully closed
    sleep(Duration::from_secs(1)).await;

    // Start VSCode again (macOS-specific)
    let open_status = Command::new("open")
        .arg("-a")
        .arg("Visual Studio Code")
        .status()
        .await?;

    if open_status.success() {
        println!("\x1b[92m[VSCode]: Restarted successfully\x1b[0m");
    } else {
        eprintln!("\x1b[91m[VSCode]: Failed to restart\x1b[0m");
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

pub async fn say_send(sender: &str, prompt: &str) -> anyhow::Result<()> {
    let prefix = "Сообщение от ";
    let prompt = format!("{prefix} {sender} {prompt}");
    Command::new("say")
        .arg("-v")
        .arg("Milena")
        .arg("-r")
        .arg("125")
        .arg(prompt)
        .status()
        .await?;
    Ok(())
}

pub async fn test_send_to_terminal(sender: &str, prompt: &str) -> io::Result<()> {
    println!("\nStep 1: Clearing and starting Gemini");
    tokio::time::sleep(Duration::from_secs(2)).await;
    say_send(sender, prompt).await.unwrap();

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
