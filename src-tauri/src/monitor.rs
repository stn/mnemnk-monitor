use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    io::{stdin, BufRead, BufReader},
    sync::Mutex,
    time::SystemTime,
};
use tauri::{AppHandle, Emitter, Manager};

const AGENT_NAME: &str = "mnemnk-monitor";
const EMIT_INPUT: &str = "mnemnk-input";

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct AgentConfig {}

impl From<&str> for AgentConfig {
    fn from(_s: &str) -> Self {
        let config = AgentConfig::default();
        config
    }
}

pub fn init(app_handle: &AppHandle, config: Option<String>) {
    let config = config.map_or_else(AgentConfig::default, |s| s.as_str().into());
    app_handle.manage(Mutex::new(config));
}

pub fn run(app_handle: &AppHandle) -> Result<()> {
    log::info!("Starting {}.", AGENT_NAME);

    let app_handle = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        let mut reader = BufReader::new(stdin());
        let mut line = String::new();

        // Main loop with graceful shutdown
        loop {
            let num = reader.read_line(&mut line).unwrap();
            if num == 0 {
                break;
            }
            if let Err(e) = process_line(&app_handle, &line).await {
                log::error!("Failed to process line: {}", e);
            }
            line.clear();
        }
    });
    Ok(())
}

async fn process_line(app_handle: &AppHandle, line: &str) -> Result<()> {
    // log::debug!("process_line: {}", line);

    if let Some((cmd, args)) = parse_line(line) {
        match cmd {
            ".CONFIG" => {
                // log::debug!("CONFIG {}.", args);
                let config: AgentConfig = args.into();
                app_handle.manage(Mutex::new(config));
            }
            ".IN" => {
                // log::debug!("IN {}.", args);
                let event = parse_input(args);
                if let Some(event) = event {
                    app_handle.emit(EMIT_INPUT, event)?;
                }
            }
            ".QUIT" => {
                log::info!("Quit {}.", AGENT_NAME);
                std::process::exit(0);
            }
            _ => {
                log::error!("Unknown command: {}", cmd);
            }
        }
    }
    Ok(())
}

fn parse_line(line: &str) -> Option<(&str, &str)> {
    if line.is_empty() {
        return None;
    }

    let line = line.trim();
    if line.is_empty() {
        return None;
    }

    if let Some((cmd, args)) = line.split_once(" ") {
        Some((cmd, args))
    } else {
        Some((line, ""))
    }
}

#[derive(Debug, Clone, Serialize)]
struct InputEvent {
    agent: String,
    channel: String,
    value: Value,
    time: u128,
}

fn parse_input(args: &str) -> Option<InputEvent> {
    let args: Vec<&str> = args.splitn(3, ' ').collect();
    if args.len() != 3 {
        return None;
    }
    Some(InputEvent {
        agent: args[0].to_string(),
        channel: args[1].to_string(),
        value: serde_json::from_str(args[2]).unwrap_or(Value::Null),
        time: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis(),
    })
}

#[derive(Debug, Serialize)]
struct UserMessage {
    message: String,
}

#[tauri::command]
pub fn send_message(message: String) -> Result<(), String> {
    let user_message = UserMessage { message };
    println!(
        ".OUT {} {}",
        "user_message",
        serde_json::to_string(&user_message).unwrap()
    );

    Ok(())
}
