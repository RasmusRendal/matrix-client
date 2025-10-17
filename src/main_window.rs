use std::rc::Rc;
use std::sync::Arc;

use matrix_sdk::{config::SyncSettings, ruma::events::room::message::SyncRoomMessageEvent};
use slint::{Model as _, ModelRc, ToSharedString as _, VecModel};
use tokio::runtime::Runtime;

slint::include_modules!();

pub fn run_main_window(rt: Arc<Runtime>, client: matrix_sdk::Client) {
    let window = MainWindow::new().unwrap();
    let weak: slint::Weak<MainWindow> = window.as_weak();

    // let model = ModelRc::new(VecModel::from_slice(&[]));
    // window.set_list_of_messages(model);

    client.add_event_handler(|ev: SyncRoomMessageEvent| async move {
        if let Some(event) = ev.as_original() {
            let body = event.content.body().to_owned();
            let message = MessageModel {
                sender: event.sender.to_shared_string(),
                message: body.into(),
            };
            weak.upgrade_in_event_loop(move |window2| {
                let mut messages: Vec<MessageModel> =
                    window2.get_list_of_messages().iter().collect();
                messages.push(message);
                window2.set_list_of_messages(ModelRc::new(VecModel::from(messages)));
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
    // TODO: This is terrible
    Box::leak(wb);
}
