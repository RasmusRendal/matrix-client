//! Module which defines utilities for printing errors in Slint error dialogues

use slint::ToSharedString;

slint::include_modules!();

#[allow(clippy::expect_used)]
pub fn run_warning_dialog(message: String) {
    let dialog = WarningDialog::new().expect("Failed to create error dialog");
    dialog.set_message(message.to_shared_string());
    let weak = dialog.as_weak();
    dialog.on_close(move || {
        weak.upgrade_in_event_loop(move |dialog| {
            dialog.hide().expect("Failed to close error dialog");
        })
        .expect("Failed closing error dialog. Event loop must be already closed.");
    });
    dialog.show().expect("Failed to show error dialog");
    Box::leak(Box::new(dialog));
}

#[allow(clippy::expect_used)]
pub fn run_or_error<F, R>(fun: F) -> Option<R>
where
    F: FnOnce() -> anyhow::Result<R>,
{
    match fun() {
        Ok(r) => Some(r),
        Err(err) => {
            slint::invoke_from_event_loop(move || {
                run_warning_dialog(err.to_string());
            })
            .expect("Failed showing error dialog. The event loop must already have closed.");
            None
        }
    }
}

#[allow(clippy::expect_used)]
pub async fn async_run_or_error<F, R, Fut>(fun: F) -> Option<R>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = anyhow::Result<R>>,
{
    match fun().await {
        Ok(r) => Some(r),
        Err(err) => {
            slint::invoke_from_event_loop(move || {
                run_warning_dialog(err.to_string());
            })
            .expect("Failed showing error dialog. The event loop must already have closed.");
            None
        }
    }
}
