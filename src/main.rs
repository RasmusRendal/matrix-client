mod client;
mod homeserver_selector;
mod main_window;
mod password_login;
mod warning_dialog;

use std::sync::Arc;

use homeserver_selector::start_select_homeserver_window;

use crate::{client::get_matrix_client, main_window::run_main_window};

slint::include_modules!();

fn main() -> anyhow::Result<()> {
    println!("main: we are in thread {:?}", std::thread::current().id());
    let rt = Arc::new(
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap(),
    );
    let startup_window = StartupWindow::new()?;
    let startup_window_weak = startup_window.as_weak();
    startup_window.show()?;
    let rt2 = rt.clone();
    slint::spawn_local(async move {
        let client = rt2.spawn(get_matrix_client()).await.unwrap().unwrap();
        match client {
            Some(client) => {
                slint::invoke_from_event_loop(|| run_main_window(rt2, client)).unwrap();
            }
            None => {
                start_select_homeserver_window(rt2).unwrap();
            }
        }
        startup_window_weak
            .upgrade_in_event_loop(|w| w.hide().unwrap())
            .unwrap();
    })
    .unwrap();
    slint::run_event_loop()?;
    println!("metrics: {:?}", rt.metrics());
    Ok(())
}
