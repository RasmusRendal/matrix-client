use std::{
    fs::File,
    io::{BufReader, Read as _},
    path::PathBuf,
};

use anyhow::Context as _;
use matrix_sdk::authentication::matrix::MatrixSession;
use serde::{Deserialize, Serialize};

/// The full session to persist.
/// Copied from https://github.com/matrix-org/matrix-rust-sdk/blob/a4e68ba8857af516b6c06387d0bfbc7a7d1fb5bf/examples/persist_session/src/main.rs
/// Men modificeret
#[derive(Debug, Serialize, Deserialize)]
struct FullSession {
    homeserver_url: String,

    /// The Matrix user session.
    user_session: MatrixSession,

    /// The latest sync token.
    ///
    /// It is only needed to persist it when using `Client::sync_once()` and we
    /// want to make our syncs faster by not receiving all the initial sync
    /// again.
    #[serde(skip_serializing_if = "Option::is_none")]
    sync_token: Option<String>,
}

fn store_dir() -> PathBuf {
    PathBuf::from("/home/rasmus/.rasmus-matrix-client/")
}

fn get_session_file() -> anyhow::Result<PathBuf> {
    Ok(store_dir().join("session_file.json"))
}

fn get_saved_session() -> anyhow::Result<Option<FullSession>> {
    let session_file = get_session_file()?;
    if !session_file.exists() {
        return Ok(None);
    }
    let contents = std::fs::read_to_string(session_file)?;
    Ok(Some(
        serde_json::from_str(&contents).context("Tried to return full session")?,
    ))
}

pub fn build_matrix_client() -> matrix_sdk::ClientBuilder {
    let store_dir = store_dir();
    matrix_sdk::Client::builder().sqlite_store(store_dir, None)
}

pub async fn get_matrix_client() -> anyhow::Result<Option<matrix_sdk::Client>> {
    let store_dir = store_dir();
    if !store_dir.exists() {
        return Ok(None);
    }
    if let Some(FullSession {
        homeserver_url,
        user_session,
        sync_token: _,
    }) = get_saved_session()?
    {
        // Build the client with the previous settings from the session.
        let client = matrix_sdk::Client::builder()
            .homeserver_url(homeserver_url)
            .sqlite_store(store_dir, None)
            .build()
            .await?; // Restore the Matrix user session.
        client.restore_session(user_session).await?;
        return Ok(Some(client));
    }
    Ok(None)
}

pub async fn save_matrix_session(client: &matrix_sdk::Client) -> anyhow::Result<()> {
    let user_session = client
        .matrix_auth()
        .session()
        .expect("A logged-in client should have a session");
    let serialized_session = FullSession {
        homeserver_url: client.homeserver().to_string(),
        user_session,
        sync_token: None,
    };
    tokio::fs::write(
        get_session_file()?,
        serde_json::to_string(&serialized_session)?,
    )
    .await?;
    Ok(())
}
