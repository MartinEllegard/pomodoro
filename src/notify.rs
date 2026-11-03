use notify_rust::Notification;

/// Show a desktop notification without blocking the event loop.
///
/// Fire-and-forget: a missing notification daemon or D-Bus session must never
/// take the timer down, so errors are deliberately ignored.
pub fn spawn(summary: String, body: String) {
    tokio::spawn(async move {
        let _ = Notification::new()
            .appname("pomodoro")
            .summary(&summary)
            .body(&body)
            .show_async()
            .await;
    });
}
