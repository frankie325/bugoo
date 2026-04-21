use std::sync::Arc;
use tokio::time::{interval, Duration};
use crate::db::Database;
use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

pub async fn start_notification_scheduler(db: Arc<Database>, app: AppHandle) {
    let mut ticker = interval(Duration::from_secs(3600)); // Check every hour

    loop {
        ticker.tick().await;
        check_and_send_notifications(&db, &app).await;
    }
}

async fn check_and_send_notifications(db: &Arc<Database>, app: &AppHandle) {
    let now = chrono::Utc::now().timestamp();

    // Scope guard to end before we collect rows
    let words: Vec<(String, String, String)> = {
        let guard = db.conn.lock().await;
        let mut stmt = match guard.prepare("SELECT id, word, translation FROM words WHERE status = 'learning' AND next_review_at <= ? LIMIT 5") {
            Ok(s) => s,
            Err(e) => {
                log::error!("prepare failed: {}", e);
                return;
            }
        };
        let rows = stmt.query_map([now], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)));
        match rows {
            Ok(r) => r.filter_map(|r| r.ok()).collect(),
            Err(e) => {
                log::error!("query failed: {}", e);
                return;
            }
        }
    }; // guard and stmt dropped here

    for (word_id, word, translation) in words {
        send_review_notification(app, &word_id, &word, &translation);
    }
}

fn send_review_notification(app: &AppHandle, _word_id: &str, word: &str, translation: &str) {
    log::info!("复习提醒: {} - {}", word, translation);
    // Send system notification
    if let Err(e) = app.notification()
        .builder()
        .title("布谷鸟 - 复习提醒")
        .body(&format!("{}: {}", word, translation))
        .show() {
        log::error!("Failed to send notification: {}", e);
    }
}
