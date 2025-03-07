use anyhow::Result;
use schemars::{schema_for, JsonSchema};
use serde::Serialize;
use serde_json::Value;
use std::{
    io::{stdin, BufRead, BufReader},
    time::SystemTime,
};
use tauri::{AppHandle, Emitter};

const AGENT_NAME: &str = "mnemnk-monitor";
const EMIT_PUBLISH: &str = "mnemnk-publish";

/// # Mnemnk Monitor
/// Monitor channels
#[derive(Debug, serde::Deserialize, serde::Serialize, JsonSchema)]
pub struct AgentConfig {
    /// # Subscribe Channels
    monitor_channels: Vec<String>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            monitor_channels: vec!["application".into(), "browser".into()],
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
        }
        config
    }
}

pub struct MonitorAgent {
    app_hanlder: AppHandle,
    config: AgentConfig,
}

impl MonitorAgent {
    pub fn new(app_handle: AppHandle, config: Option<String>) -> Self {
        Self {
            app_hanlder: app_handle,
            config: config.map_or_else(AgentConfig::default, |s| s.as_str().into()),
        }
    }

    pub fn run(self) -> Result<()> {
        log::info!("Starting {}.", AGENT_NAME);

        let schema = schema_for!(AgentConfig);
        println!(".CONFIG_SCHEMA {}", serde_json::to_string(&schema)?);
        println!(".CONFIG {}", serde_json::to_string(&self.config)?);

        for channel in &self.config.monitor_channels {
            println!(".SUBSCRIBE {}", channel);
        }

        tauri::async_runtime::spawn(async move {
            let mut reader = BufReader::new(stdin());
            let mut line = String::new();

            // Main loop with graceful shutdown
            loop {
                let num = reader.read_line(&mut line).unwrap();
                if num == 0 {
                    break;
                }
                if let Err(e) = self.process_line(&line).await {
                    log::error!("Failed to process line: {}", e);
                }
                line.clear();
            }
        });
        Ok(())
    }

    async fn process_line(&self, line: &str) -> Result<()> {
        log::debug!("process_line: {}", line);

        if let Some((cmd, args)) = parse_line(line) {
            match cmd {
                ".PUBLISH" => {
                    log::debug!("PUBLISH {}.", args);
                    let event = parse_publish(args);
                    if let Some(event) = event {
                        log::debug!("PUBLISH {:?}", event);
                        self.app_hanlder.emit(EMIT_PUBLISH, event)?;
                    }
                }
                ".QUIT" => {
                    log::info!("QUIT {}.", AGENT_NAME);
                    std::process::exit(0);
                }
                _ => {
                    log::error!("Unknown command: {}", cmd);
                }
            }
        }
        Ok(())
    }
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
