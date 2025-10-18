use std::rc::Rc;
use std::sync::Arc;

use anyhow::Context;
use matrix_sdk::{
    config::SyncSettings,
    deserialized_responses::TimelineEvent,
    ruma::{
        OwnedRoomId, RoomId,
        events::room::message::{RoomMessageEventContent, SyncRoomMessageEvent},
    },
    stream::StreamExt,
};
use matrix_sdk_ui::{
    eyeball_im::{Vector, VectorDiff},
    timeline::{RoomExt, TimelineItem},
};
use slint::{Model as _, ModelRc, SharedString, ToSharedString as _, VecModel};
use tokio::{runtime::Runtime, sync::Mutex, task::JoinHandle};

slint::include_modules!();

#[derive(Clone)]
struct SendableModel {
    room_id: OwnedRoomId,
    messages: Vector<Arc<TimelineItem>>,
}

impl From<SendableModel> for VisibleRoomModel {
    fn from(val: SendableModel) -> Self {
        let messages: Vec<MessageModel> = val
            .messages
            .into_iter()
            .filter_map(|e| {
                if let Some(e) = e.as_event()
                    && let Some(message_content) = e.content().as_message()
                {
                    Some(MessageModel {
                        message: message_content.body().to_shared_string(),
                        sender: e.sender().to_shared_string(),
                    })
                } else {
                    None
                }
            })
            .collect();
        VisibleRoomModel {
            room_id: val.room_id.to_shared_string(),
            messages: ModelRc::new(VecModel::from_slice(&messages)),
        }
    }
}

fn apply_changes(
    model: Arc<std::sync::Mutex<Option<SendableModel>>>,
    main_window: MainWindow,
    changes: Vec<VectorDiff<Arc<TimelineItem>>>,
) {
    let mut lock = model.lock().unwrap();
    let mut model = lock.take().unwrap();
    for change in changes {
        change.apply(&mut model.messages);
    }
    main_window.set_visible_room(model.clone().into());
    *lock = Some(model);
}

async fn construct_room_view(
    weak: slint::Weak<MainWindow>,
    timeline: Arc<std::sync::Mutex<Option<SendableModel>>>,
    stream_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
    client: matrix_sdk::Client,
    room_id: OwnedRoomId,
) -> anyhow::Result<()> {
    println!("Building the room view");
    let room = client
        .get_room(&room_id)
        .context("Tried to get a non-existent room")?;
    let roomtimeline = room.timeline().await.context("Failed to get timeline")?;
    roomtimeline.paginate_forwards(20).await?;
    let (timeline_items, mut stream) = roomtimeline.subscribe().await;
    {
        let mut lock = timeline.lock().unwrap();
        *lock = Some(SendableModel {
            room_id: room_id.clone(),
            messages: timeline_items,
        });
    }
    let timeline2 = timeline.clone();
    weak.upgrade_in_event_loop(move |w| {
        let mut lock = timeline2.lock().unwrap();
        if let Some(timeline) = lock.take() {
            w.set_visible_room(timeline.clone().into());
            *lock = Some(timeline);
        }
    })
    .unwrap();
    {
        let mut lock = stream_handle.lock().await;
        if let Some(handle) = lock.take() {
            handle.abort();
        }
        let handle = tokio::spawn(async move {
            while let Some(messages) = stream.next().await {
                let timeline = timeline.clone();
                weak.upgrade_in_event_loop(move |main_window| {
                    apply_changes(timeline.clone(), main_window, messages);
                })
                .unwrap();
            }
        });
        *lock = Some(handle);
    }
    println!("got the timeline: {roomtimeline:?}");
    Ok(())
}

pub fn run_main_window(rt: Arc<Runtime>, client: matrix_sdk::Client) {
    let window = MainWindow::new().unwrap();
    let stream_handle: Arc<Mutex<Option<JoinHandle<()>>>> = Arc::new(Mutex::new(None));
    let model: Arc<std::sync::Mutex<Option<SendableModel>>> = Arc::new(std::sync::Mutex::new(None));

    let v: Vec<_> = client
        .rooms()
        .iter()
        .map(|r| RoomModel {
            id: r.room_id().to_string().into(),
            name: r
                .name()
                .unwrap_or(r.room_id().to_string())
                .to_shared_string(),
        })
        .collect();
    window.set_list_of_rooms(ModelRc::new(VecModel::from(v)));

    let weak = window.as_weak();
    let client2 = client.clone();
    let rt2 = rt.clone();
    let model2 = model.clone();
    window.on_show_room(move |sp: SharedString| {
        let room_id: OwnedRoomId = sp.parse().unwrap();
        let weak = weak.clone();
        let client2 = client2.clone();
        let rt2 = rt2.clone();
        let stream_handle = stream_handle.clone();
        let model2 = model2.clone();
        slint::spawn_local(async move {
            rt2.spawn(async_compat::Compat::new(construct_room_view(
                weak.clone(),
                model2.clone(),
                stream_handle.clone(),
                client2,
                room_id,
            )))
            .await
            .unwrap()
            .unwrap();
        })
        .unwrap();
    });

    let model2 = model.clone();
    let client2 = client.clone();
    let rt2 = rt.clone();
    window.on_send_message(move |message: SharedString| {
        let room_id = model2.lock().unwrap().as_ref().unwrap().room_id.clone();
        let client2 = client2.clone();
        rt2.spawn(async move {
            let room_id = room_id.clone();
            let client2 = client2.clone();

            client2
                .get_room(&room_id)
                .unwrap()
                .send(RoomMessageEventContent::text_plain(message))
                .await
                .unwrap();
        });
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
