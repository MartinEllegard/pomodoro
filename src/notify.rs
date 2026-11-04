use notify_rust::Notification;

/// Show a desktop notification without blocking the event loop.
///
/// Fire-and-forget: a missing notification daemon, D-Bus session or
/// notification permission must never take the timer down, so errors are
/// deliberately ignored.
///
/// Linux/BSD: zbus is async, so the request runs as a tokio task.
/// macOS/Windows: `show` is blocking, so it runs on a detached thread that the
/// process never joins. A thread still waiting for macOS's delivery
/// confirmation (up to 2 s) simply dies with the process at exit.
pub fn spawn(summary: String, body: String) {
    #[cfg(all(unix, not(target_os = "macos")))]
    tokio::spawn(async move {
        let _ = notification(&summary, &body).show_async().await;
    });

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    std::thread::spawn(move || {
        let _ = notification(&summary, &body).show();
    });
}

fn notification(summary: &str, body: &str) -> Notification {
    let mut notification = Notification::new();
    notification.appname("pomodoro").summary(summary).body(body);
    notification
}
