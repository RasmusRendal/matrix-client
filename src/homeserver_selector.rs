use slint::ComponentHandle;
use tokio::sync::mpsc;

use crate::{client::build_matrix_client, password_login::start_password_window};

slint::include_modules!();

async fn try_login(mut channel: mpsc::Receiver<String>) -> matrix_sdk::Client {
    while let Some(homeserver_url) = channel.recv().await {
        let c = build_matrix_client()
            .server_name_or_homeserver_url(homeserver_url)
            .build()
            .await;
        if let Ok(c) = c {
            return c;
        }
    }
    panic!("oh no");
}

pub fn start_select_homeserver_window() -> anyhow::Result<()> {
    let ui = LoginWindow::new()?;

    let (tx, rx) = mpsc::channel::<String>(100);

    ui.on_connect({
        move |homeserver_url: slint::SharedString| {
            let tx = tx.clone();
            slint::spawn_local(async move {
                tx.send(homeserver_url.to_string()).await.unwrap();
            })
            .unwrap();
        }
    });

    ui.show()?;
    slint::spawn_local(async move {
        let client = async_compat::Compat::new(try_login(rx)).await;
        let login_types = async_compat::Compat::new(client.matrix_auth().get_login_types())
            .await
            .unwrap();
        println!("we have a client! {:?}", client);
        println!("Login type: {:?}", login_types);
        start_password_window(client).unwrap();
        ui.hide().unwrap();
    })?;

    Ok(())
}
