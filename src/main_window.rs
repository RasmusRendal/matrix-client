use std::{sync::Arc, time::Duration};

use matrix_sdk::{config::SyncSettings, ruma::events::room::message::SyncRoomMessageEvent};
use tokio::runtime::Runtime;

slint::include_modules!();

pub fn run_main_window(rt: Arc<Runtime>, client: matrix_sdk::Client) {
    println!(
        "run main window: we are in thread {:?}",
        std::thread::current().id()
    );
    assert!(client.matrix_auth().logged_in());
    let window = MainWindow::new().unwrap();
    let weak: slint::Weak<MainWindow> = window.as_weak();

    client.add_event_handler(|ev: SyncRoomMessageEvent| async move {
        println!("Received a message {:?}", ev);
        if let Some(event) = ev.as_original() {
            let body = event.content.body().to_owned();
            println!(
                "event handler: we are in thread {:?}",
                std::thread::current().id()
            );
            // weak.upgrade_in_event_loop(move |window2| {
            //     println!(
            //         "writing message: we are in thread {:?}",
            //         std::thread::current().id()
            //     );

            //     window2.invoke_add_message(body.into());
            // })
            // .unwrap();
            slint::invoke_from_event_loop(move || {
                println!(
                    "event loop: we are in thread {:?}",
                    std::thread::current().id()
                );
                let window2 = weak.upgrade().unwrap();
                window2.invoke_add_message(body.into());
            })
            .unwrap();
        }
    });

    rt.spawn(async move {
        assert!(client.matrix_auth().logged_in());
        client.sync(SyncSettings::default()).await.unwrap();
    });
    window.show().unwrap();
    let wb = Box::new(window);
    Box::leak(wb);
}
