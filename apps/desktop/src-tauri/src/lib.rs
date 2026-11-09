#[tauri::command]
fn application_status() -> &'static str {
    "KRAQEL 0.1 foundation"
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
/// Startet die Desktop-Anwendung.
///
/// # Panics
///
/// Bricht ab, wenn die Tauri-Laufzeit nicht initialisiert oder ausgeführt werden kann.
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![application_status])
        .run(tauri::generate_context!())
        .expect("KRAQEL konnte nicht gestartet werden");
}

#[cfg(test)]
mod tests {
    use super::application_status;

    #[test]
    fn status_identifies_the_foundation() {
        assert_eq!(application_status(), "KRAQEL 0.1 foundation");
    }
}
