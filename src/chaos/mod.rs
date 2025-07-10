use serde_json::Value;
use std::{env, fs};
use tracing::info;

pub fn overwrite_custom_css(mode: &str) -> anyhow::Result<()> {
    let username = env::var("SYS_USER").unwrap();
    let file_path = format!(
        "/Users/{}/Library/Application Support/Code/User/settings.json",
        username
    );

    let json_content = fs::read_to_string(&file_path).unwrap();
    let mut json: Value = json5::from_str(&json_content).unwrap();

    info!("Mode {}", mode.trim());
    let new_imports = vec![Value::String(format!(
        "file:///Users/{}/Desktop/chat-tvari/{}.css",
        username,
        mode.trim()
    ))];

    json["vscode_custom_css.imports"] = Value::Array(new_imports);

    // Write it back, pretty-printed
    fs::write(&file_path, serde_json::to_string_pretty(&json)?)?;

    println!("Updated vscode_custom_css.imports in {file_path}");
    Ok(())
}
