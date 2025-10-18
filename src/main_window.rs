use std::rc::Rc;
use std::sync::Arc;

use matrix_sdk::{config::SyncSettings, ruma::events::room::message::SyncRoomMessageEvent};
use slint::{Model as _, ModelRc, SharedString, ToSharedString as _, VecModel};
use tokio::runtime::Runtime;

slint::include_modules!();

pub fn run_main_window(rt: Arc<Runtime>, client: matrix_sdk::Client) {
    let window = MainWindow::new().unwrap();
    let weak: slint::Weak<MainWindow> = window.as_weak();

    client.add_event_handler(|ev: SyncRoomMessageEvent| async move {
        if let Some(event) = ev.as_original() {
            let body = event.content.body().to_owned();
            let message = MessageModel {
                sender: event.sender.to_shared_string(),
                message: body.into(),
            };
            weak.upgrade_in_event_loop(move |window2| {
                let mut room_model: VisibleRoomModel = window2.get_visible_room();
                let mut messages: Vec<MessageModel> = room_model.messages.iter().collect();
                messages.push(message);
                room_model.messages = ModelRc::new(VecModel::from(messages));
                window2.set_visible_room(room_model);
                // window2.set_visible_rooms(ModelRc::new(VecModel::from(messages)));
            })
            .unwrap();
        }
    });

    let v: Vec<_> = client
        .rooms()
        .iter()
        .map(|r| RoomModel {
            id: r.room_id().to_string().into(),
        })
        .collect();
    window.set_list_of_rooms(ModelRc::new(VecModel::from(v)));

    window.on_show_room(|sp: SharedString| {
        println!("{sp}");
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
