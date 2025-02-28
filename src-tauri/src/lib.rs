use tauri_plugin_cli::CliExt;

mod monitor;
use monitor::MonitorAgent;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_cli::init())
        .setup(|app| {
            let mut config = None;
            match app.cli().matches() {
                // `matches` here is a Struct with { args, subcommand }.
                // `args` is `HashMap<String, ArgData>` where `ArgData` is a struct with { value, occurrences }.
                // `subcommand` is `Option<Box<SubcommandMatches>>` where `SubcommandMatches` is a struct with { name, matches }.
                Ok(matches) => {
                    println!("{:?}", matches);
                    if let Some(config_arg) = matches.args.get("config") {
                        config = config_arg.value.as_str().map(|s| s.to_string());
                    }
                }
                Err(_) => {
                    println!("No matches");
                }
            }
            MonitorAgent::new(app.handle().clone(), config)
                .run()
                .unwrap();
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
