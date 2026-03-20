use crate::ui_cm_interface::{Client, InvokeUiCM};
use hbb_common::log;

/// Minimal connection manager handler for compass-rmm headless mode.
/// Satisfies the InvokeUiCM trait so the --server child process can
/// connect to the CM pipe without errors. No UI, no stdout output.
#[derive(Clone)]
pub struct HeadlessCmHandler;

impl InvokeUiCM for HeadlessCmHandler {
    fn add_connection(&self, client: &Client) {
        log::info!("Session started: peer={}, id={}", client.peer_id, client.id);
    }

    fn remove_connection(&self, id: i32, _close: bool) {
        log::info!("Session ended: id={}", id);
    }

    fn new_message(&self, _id: i32, _text: String) {}
    fn change_theme(&self, _dark: String) {}
    fn change_language(&self) {}
    fn show_elevation(&self, _show: bool) {}
    fn update_voice_call_state(&self, _client: &Client) {}
    fn file_transfer_log(&self, _action: &str, _log: &str) {}
}
