use crate::ui_cm_interface::{Client, InvokeUiCM};

#[derive(Clone)]
pub struct HeadlessCmHandler;

impl InvokeUiCM for HeadlessCmHandler {
    fn add_connection(&self, client: &Client) {
        let event = serde_json::json!({
            "event": "session_start",
            "id": client.id,
            "peer_id": client.peer_id,
            "name": client.name,
            "authorized": client.authorized,
            "is_file_transfer": client.is_file_transfer,
            "keyboard": client.keyboard,
            "clipboard": client.clipboard,
            "audio": client.audio,
        });
        println!("{}", event);

        #[cfg(target_os = "windows")]
        {
            let peer = if client.name.is_empty() {
                client.peer_id.clone()
            } else {
                format!("{} ({})", client.name, client.peer_id)
            };
            let _ = tauri_winrt_notification::Toast::new(tauri_winrt_notification::Toast::POWERSHELL_APP_ID)
                .title("Compass RMM")
                .text1(&format!("Remote session from {}", peer))
                .show();
        }
    }

    fn remove_connection(&self, id: i32, close: bool) {
        let event = serde_json::json!({
            "event": "session_end",
            "id": id,
            "close": close,
        });
        println!("{}", event);
    }

    fn new_message(&self, id: i32, text: String) {
        let event = serde_json::json!({
            "event": "message",
            "id": id,
            "text": text,
        });
        println!("{}", event);
    }

    fn change_theme(&self, _dark: String) {}

    fn change_language(&self) {}

    fn show_elevation(&self, _show: bool) {}

    fn update_voice_call_state(&self, _client: &Client) {}

    fn file_transfer_log(&self, action: &str, log: &str) {
        let event = serde_json::json!({
            "event": "file_transfer",
            "action": action,
            "log": log,
        });
        println!("{}", event);
    }
}
