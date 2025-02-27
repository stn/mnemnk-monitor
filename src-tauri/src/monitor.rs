use anyhow::Result;
use schemars::{schema_for, JsonSchema};
use serde_json::Value;
use tokio::io::{stdin, AsyncBufReadExt, BufReader};

const AGENT_NAME: &str = "mnemnk-monitor";

/// # Monitor
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
    config: AgentConfig,
}

impl MonitorAgent {
    pub fn new(config: Option<String>) -> Self {
        Self {
            config: config.map_or_else(AgentConfig::default, |s| s.as_str().into()),
        }
    }

    pub fn run(self) -> Result<()> {
        let schema = schema_for!(AgentConfig);
        println!("CONFIG_SCHEMA {}", serde_json::to_string(&schema)?);
        println!("CONFIG {}", serde_json::to_string(&self.config)?);

        log::info!("Starting {}.", AGENT_NAME);

        tauri::async_runtime::spawn(async move {
            let mut reader = BufReader::new(stdin());
            let mut line = String::new();

            // Main loop with graceful shutdown
            loop {
                reader.read_line(&mut line).await.unwrap();
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

        if let Some((cmd, _args)) = parse_line(line) {
            match cmd {
                "QUIT" => {
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
