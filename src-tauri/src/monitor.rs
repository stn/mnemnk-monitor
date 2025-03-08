use anyhow::Result;
use schemars::{schema_for, JsonSchema};
use serde::Serialize;
use serde_json::Value;
use std::{
    io::{stdin, BufRead, BufReader},
    sync::Mutex,
    time::SystemTime,
};
use tauri::{AppHandle, Emitter, Manager, State};

const AGENT_NAME: &str = "mnemnk-monitor";
const EMIT_PUBLISH: &str = "mnemnk-publish";

/// # Mnemnk Monitor
/// Monitor channels
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, JsonSchema)]
pub struct AgentConfig {
    /// # Subscribe Channels
    monitor_channels: Vec<String>,
    /// # Message channel
    message_channel: String,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            monitor_channels: vec!["application".into(), "browser".into()],
            message_channel: "user_message".into(),
        }
    }
}

impl From<&str> for AgentConfig {
    fn from(s: &str) -> Self {
        let mut config = AgentConfig::default();
        if let Value::Object(c) = serde_json::from_str(s).unwrap_or(Value::Null) {
            if let Some(channels) = c.get("monitor_channels") {
                config.monitor_channels = channels
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_str().unwrap().to_string())
                    .collect();
            }
            if let Some(message_channel) = c.get("message_channel") {
                config.message_channel = message_channel.as_str().unwrap().to_string();
            }
        }
        config
    }
}

pub fn init(app_handle: &AppHandle, config: Option<String>) {
    let config = config.map_or_else(AgentConfig::default, |s| s.as_str().into());
    app_handle.manage(Mutex::new(config));
}

pub fn run(app_handle: &AppHandle) -> Result<()> {
    log::info!("Starting {}.", AGENT_NAME);

    let schema = schema_for!(AgentConfig);
    println!(".CONFIG_SCHEMA {}", serde_json::to_string(&schema)?);

    let config = app_handle.state::<Mutex<AgentConfig>>();
    {
        let config = config.lock().unwrap().clone();
        println!(".CONFIG {}", serde_json::to_string(&config)?);

        for channel in &config.monitor_channels {
            println!(".SUBSCRIBE {}", channel);
        }
    }

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
            ".PUBLISH" => {
                // log::debug!("PUBLISH {}.", args);
                let event = parse_publish(args);
                if let Some(event) = event {
                    log::debug!("PUBLISH {:?}", event);
                    app_handle.emit(EMIT_PUBLISH, event)?;
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
struct PublishEvent {
    agent: String,
    channel: String,
    value: Value,
    time: u128,
}

fn parse_publish(args: &str) -> Option<PublishEvent> {
    let args: Vec<&str> = args.splitn(3, ' ').collect();
    if args.len() != 3 {
        return None;
    }
    Some(PublishEvent {
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
pub fn send_message(message: String, config: State<Mutex<AgentConfig>>) -> Result<(), String> {
    let message_channel;
    {
        let config = config.lock().unwrap().clone();
        message_channel = config.message_channel;
    }

    if message_channel.is_empty() {
        return Err("Message channel is empty.".to_string());
    }

    let user_message = UserMessage { message };
    println!(
        ".WRITE {} {}",
        message_channel,
        serde_json::to_string(&user_message).unwrap()
    );

    Ok(())
}
