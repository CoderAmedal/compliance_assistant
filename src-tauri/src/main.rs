#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod commands;

use compliance_assistant::{AppState, init_app};

#[tokio::main]
async fn main() {
    let state = init_app().await.expect("Failed to initialize app");
    
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(state)
.invoke_handler(tauri::generate_handler![
            commands::get_sessions,
            commands::create_session,
            commands::delete_session,
            commands::update_session_title,
            commands::get_messages,
            commands::send_message,
            commands::list_documents,
            commands::add_document,
            commands::delete_document,
            commands::search_knowledge,
            commands::list_tools,
            commands::execute_tool,
            commands::get_tool_permissions,
            commands::set_tool_permission,
            commands::get_chat_model_config,
            commands::get_embedding_model_config,
            commands::update_chat_model_config,
            commands::update_embedding_model_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}