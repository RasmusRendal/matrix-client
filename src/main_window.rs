use matrix_sdk::config::SyncSettings;

slint::include_modules!();

pub fn run_main_window(client: matrix_sdk::Client) {
    assert!(client.matrix_auth().logged_in());

    slint::spawn_local(async move {
        async_compat::Compat::new(client.sync(SyncSettings::default()))
            .await
            .unwrap();
    })
    .unwrap();
    let window = MainWindow::new().unwrap();
    window.show().unwrap();
}
