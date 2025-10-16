use std::path::PathBuf;

fn store_dir() -> PathBuf {
    PathBuf::from("/home/rasmus/.rasmus-matrix-client/")
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
    Ok(Some(build_matrix_client().build().await?))
}
