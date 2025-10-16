use tokio::sync::mpsc;

slint::include_modules!();

async fn try_login(
    mut channel: mpsc::Receiver<(String, String)>,
    client: matrix_sdk::Client,
) -> matrix_sdk::Client {
    while let Some((username, password)) = channel.recv().await {
        let login_attempt = client
            .matrix_auth()
            .login_username(username, &password)
            .await;
        println!("Login result: {:?}", login_attempt);
        if let Ok(login_attempt) = login_attempt {
            return client;
        }
    }
    panic!("oh no");
}

pub fn start_password_window(client: matrix_sdk::Client) -> anyhow::Result<()> {
    let (tx, rx) = mpsc::channel::<(String, String)>(100);
    let password = PasswordLoginWindow::new()?;

    password.on_login(
        move |username: slint::SharedString, password: slint::SharedString| {
            let tx = tx.clone();
            slint::spawn_local(async move {
                tx.send((username.to_string(), password.to_string()))
                    .await
                    .unwrap();
            })
            .unwrap();
        },
    );

    slint::spawn_local(async move {
        let client = async_compat::Compat::new(try_login(rx, client)).await;
        let login_types = async_compat::Compat::new(client.matrix_auth().get_login_types())
            .await
            .unwrap();
        println!("we have a client! {:?}", client);
        println!("Login type: {:?}", login_types);
        start_password_window(client).unwrap();
    })?;
    password.show()?;
    Ok(())
}
