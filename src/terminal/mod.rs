use regex::Regex;
use std::env;
use std::io::{self, Write};
use std::time::Duration;
use tokio::process::Command;
use tokio::task::JoinHandle;

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

pub async fn stream_by_polling(window: usize, pane: usize) -> io::Result<()> {
    let session_name = env::var("TMUX_SESSION").unwrap();
    let target = format!("{}:{}.{}", session_name, window, pane);
    let mut last_output = String::new();

    loop {
        let mut out = Command::new("tmux")
            .args(&["capture-pane", "-p", "-t", &target])
            .output()
            .await?;

        let text = String::from_utf8_lossy(&out.stdout);
        // Only print the *new* part:
        print!("Text: {}", text);
        if text.contains("Yes, allow once") {
            send_enter().await?;
        }
        let re = Regex::new(r"\[FINISHED]\[RESPONSE]").unwrap();
        if re.is_match(&text) {
            return Ok(());
        }
        io::stdout().flush()?;
        last_output = text.into_owned();

        tokio::time::sleep(Duration::from_millis(200)).await;
    }
}
pub async fn test_send_to_terminal(prompt: &str) -> io::Result<()> {
    println!("\nStep 1: Clearing and starting Gemini");
    send_to_terminal("clear && gemini").await?;
    tokio::time::sleep(Duration::from_secs(2)).await;

    let logger: JoinHandle<io::Result<()>> = tokio::spawn(stream_by_polling(0, 0));

    println!("\nStep 2.5: Pressing ESC to clear any previous input");
    send_esc().await?;
    tokio::time::sleep(Duration::from_secs(2)).await;

    println!("\nStep 3: Type prompt");
    send_keystrokes_without_enter(prompt).await?;
    send_enter().await?;
    logger.await.unwrap()?;

    Ok(())
}
