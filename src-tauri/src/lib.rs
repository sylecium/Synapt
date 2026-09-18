use std::time::Duration;
use tauri::Manager;

pub mod commands;
pub mod db;
pub mod error;
pub mod honoraires;
pub mod honoraires_pdf;
pub mod models;
pub mod ntfy;
pub mod overlap;
pub mod repo;
pub mod settings;
pub mod stripe;
pub mod tva;

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
        .plugin(
            tauri_plugin_window_state::Builder::default()
                .with_state_flags(
                    tauri_plugin_window_state::StateFlags::SIZE
                        | tauri_plugin_window_state::StateFlags::POSITION
                        | tauri_plugin_window_state::StateFlags::MAXIMIZED,
                )
                .build(),
        )
        .on_window_event(|_window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
            }
        })
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            let conn = db::init_db().map_err(|e| {
                eprintln!("setup init_db: {}", e.message);
                Box::<dyn std::error::Error>::from(e.message)
            })?;
            app.manage(db::DbState::new(conn));

            let handle = app.handle().clone();
            commands::run_ntfy_sync(handle.clone());

            std::thread::spawn(move || loop {
                std::thread::sleep(Duration::from_secs(3600));
                commands::run_ntfy_sync(handle.clone());
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
            commands::tarifs_delete,
            commands::notes_list,
            commands::notes_upsert,
            commands::notes_delete,
            commands::rdv_list,
            commands::rdv_get,
            commands::rdv_create,
            commands::rdv_update,
            commands::rdv_annuler,
            commands::rdv_set_note,
            commands::rdv_dashboard,
            commands::stripe_ensure_link,
            commands::ntfy_test,
            commands::honoraires_list,
            commands::honoraires_get,
            commands::honoraires_create,
            commands::honoraires_ouvrir,
            commands::honoraires_annuler,
            commands::honoraires_rdvs_disponibles,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
