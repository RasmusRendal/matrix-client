pub fn run_main_window(client: matrix_sdk::Client) {
    assert!(client.matrix_auth().logged_in());
}
