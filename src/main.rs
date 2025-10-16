mod client;
mod homeserver_selector;
mod main_window;
mod password_login;

use homeserver_selector::start_select_homeserver_window;

use crate::{client::get_matrix_client, main_window::run_main_window};

slint::include_modules!();

fn main() -> anyhow::Result<()> {
    start_select_homeserver_window()?;
    // let startup_widow = StartupWindow::new();
    // startup_window.show();
    // slint::spawn_local(async || {

    // })
    // let client = get_matrix_client();
    // let matrix_client = tokio::task::spawn_blocking(get_matrix_client)?;
    // match matrix_client {
    //     Some(client) => {
    //         run_main_window(client);
    //     }
    //     None => ,
    // }
    slint::run_event_loop()?;
    Ok(())
}
