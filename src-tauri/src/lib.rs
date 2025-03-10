use tauri_plugin_cli::CliExt;

mod monitor;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(if cfg!(debug_assertions) {
                    log::LevelFilter::Debug
                } else {
                    log::LevelFilter::Info
                })
                .build(),
        )
        .plugin(tauri_plugin_cli::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let mut config = None;
            match app.cli().matches() {
                // `matches` here is a Struct with { args, subcommand }.
                // `args` is `HashMap<String, ArgData>` where `ArgData` is a struct with { value, occurrences }.
                // `subcommand` is `Option<Box<SubcommandMatches>>` where `SubcommandMatches` is a struct with { name, matches }.
                Ok(matches) => {
                    if let Some(config_arg) = matches.args.get("config") {
                        config = config_arg.value.as_str().map(|s| s.to_string());
                    }
                }
                Err(_) => {}
            }
            monitor::init(app.handle(), config);
            monitor::run(app.handle())?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![monitor::send_message])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
