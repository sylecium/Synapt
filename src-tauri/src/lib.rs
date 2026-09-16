use std::time::Duration;

pub mod commands;
pub mod db;
pub mod error;
pub mod models;
pub mod ntfy;
pub mod overlap;
pub mod settings;
pub mod stripe;

fn updater_plugin() -> tauri::plugin::TauriPlugin<tauri::Wry, tauri_plugin_updater::Config> {
    let mut builder = tauri_plugin_updater::Builder::new();
    if let Some(token) = option_env!("SYNAPT_UPDATER_TOKEN") {
        builder = builder
            .header("Authorization", format!("Bearer {token}"))
            .expect("en-tête Authorization updater");
    }
    builder.build()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(updater_plugin())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            if let Ok(path) = db::db_path() {
                if let Ok(conn) = db::open_file(&path) {
                    if let Err(e) = db::migrate(&conn) {
                        eprintln!("setup migrate: {}", e.message);
                    }
                }
            }

            commands::run_ntfy_sync();

            std::thread::spawn(|| loop {
                std::thread::sleep(Duration::from_secs(3600));
                commands::run_ntfy_sync();
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::settings_get,
            commands::settings_set,
            commands::clients_list,
            commands::clients_get,
            commands::clients_upsert,
            commands::clients_delete,
            commands::tarifs_list,
            commands::tarifs_upsert,
            commands::tarifs_set_actif,
            commands::notes_list,
            commands::notes_upsert,
            commands::notes_delete,
            commands::rdv_list,
            commands::rdv_get,
            commands::rdv_create,
            commands::rdv_update,
            commands::rdv_annuler,
            commands::rdv_dashboard,
            commands::stripe_ensure_link,
            commands::ntfy_test,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
