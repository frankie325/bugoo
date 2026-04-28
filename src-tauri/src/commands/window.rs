use tauri::{WebviewUrl, WebviewWindowBuilder};

#[tauri::command]
pub async fn open_float_window(app: tauri::AppHandle, text: String) -> Result<(), String> {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let label = format!("float-{}", timestamp);
    let encoded_text = urlencoding::encode(&text);
    let url = format!("index.html?text={}", encoded_text);

    log::info!("[DEBUG] Creating float window: label={}, text={}, url={}", label, text, url);

    let window = WebviewWindowBuilder::new(&app, &label, WebviewUrl::App(url.into()))
        .inner_size(320.0, 200.0)
        .always_on_top(true)
        .decorations(true)
        .skip_taskbar(true)
        .resizable(false)
        .build();

    match window {
        Ok(w) => {
            log::info!("[DEBUG] Float window created successfully, label={}", label);
            Ok(())
        }
        Err(e) => {
            log::error!("[DEBUG] Failed to create float window: {}", e);
            Err(e.to_string())
        }
    }
}
