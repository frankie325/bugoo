pub async fn start_notification_scheduler(
    _db: std::sync::Arc<crate::db::Database>,
    _app_handle: tauri::AppHandle,
) {
    log::info!("Notification scheduler started");
}
