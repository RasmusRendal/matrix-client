mod client;
mod homeserver_selector;
mod main_window;
mod password_login;

use homeserver_selector::start_select_homeserver_window;

use crate::{client::get_matrix_client, main_window::run_main_window};

slint::include_modules!();

fn main() -> anyhow::Result<()> {
    let startup_window = StartupWindow::new()?;
    startup_window.show()?;
    slint::spawn_local(async move {
        let client = async_compat::Compat::new(get_matrix_client())
            .await
            .unwrap();
        match client {
            Some(client) => {
                run_main_window(client);
            }
            None => {
                start_select_homeserver_window().unwrap();
            }
        }
        startup_window.hide().unwrap();
    })
    .unwrap();
    slint::run_event_loop()?;
    Ok(())
}
