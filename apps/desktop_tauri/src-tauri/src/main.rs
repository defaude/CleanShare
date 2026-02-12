#[tauri::command]
fn clean_text(input: String) -> String {
    link_cleaner_core::clean_text(&input)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![clean_text])
        .run(tauri::generate_context!())
        .expect("failed to run tauri app");
}
