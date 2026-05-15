use std::collections::HashMap;
use std::io;
use std::path::PathBuf;
use std::sync::Mutex;

use fm_application::DrivePort;
use fm_domain::DriveEntry;
use google_drive3::DriveHub;
use google_drive3::api::File;
use google_drive3::hyper_rustls;
use google_drive3::hyper_util;
use google_drive3::yup_oauth2::{
    InstalledFlowAuthenticator, InstalledFlowReturnMethod, read_application_secret,
};

const DRIVE_FOLDER_MIME_TYPE: &str = "application/vnd.google-apps.folder";

pub struct GoogleDriveAdapter {
    credentials_path: PathBuf,
    token_path: PathBuf,
    folder_cache: Mutex<HashMap<String, Vec<DriveEntry>>>,
}

impl GoogleDriveAdapter {
    pub fn new(credentials_path: impl Into<PathBuf>, token_path: impl Into<PathBuf>) -> Self {
        Self {
            credentials_path: credentials_path.into(),
            token_path: token_path.into(),
            folder_cache: Mutex::new(HashMap::new()),
        }
    }

    // builds authenticated Google Drive client
    async fn build_hub(
        &self,
    ) -> io::Result<
        DriveHub<hyper_rustls::HttpsConnector<hyper_util::client::legacy::connect::HttpConnector>>,
    > {
        // load OAuth client credentials
        let secret = read_application_secret(&self.credentials_path)
            .await
            .map_err(to_io_error)?;

        // create authenticator and persist tokens locally
        let auth =
            InstalledFlowAuthenticator::builder(secret, InstalledFlowReturnMethod::HTTPRedirect)
                .persist_tokens_to_disk(&self.token_path)
                .build()
                .await
                .map_err(to_io_error)?;

        // HTTPS connector for secure communication
        let connector = hyper_rustls::HttpsConnectorBuilder::new()
            .with_native_roots()
            .map_err(to_io_error)?
            .https_or_http()
            .enable_http1()
            .build();

        // HTTP client used by Drive API
        let client =
            hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
                .build(connector);

        // main Google Drive API entry point
        Ok(DriveHub::new(client, auth))
    }

    // async logic for listing folder contents
    async fn list_folder_async(&self, folder_id: &str) -> io::Result<Vec<DriveEntry>> {
        let hub = self.build_hub().await?;

        // query: all non-trashed items having this folder as parent
        let query = format!(
            "'{}' in parents and trashed = false",
            folder_id.replace('\'', "\\'")
        );

        // execute API request and extract file list
        let (_, file_list) = hub
            .files()
            .list()
            .q(&query)
            .param("fields", "files(id,name,mimeType)")
            .add_scope(google_drive3::api::Scope::Full)
            .doit()
            .await
            .map_err(to_io_error)?;

        let files = file_list.files.unwrap_or_default();

        // map API model -> domain model
        Ok(files.into_iter().filter_map(file_to_drive_entry).collect())
    }

    fn get_cached_folder(&self, folder_id: &str) -> io::Result<Option<Vec<DriveEntry>>> {
        let cache = self
            .folder_cache
            .lock()
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "Drive cache lock poisoned"))?;

        Ok(cache.get(folder_id).cloned())
    }

    fn store_cached_folder(&self, folder_id: &str, entries: Vec<DriveEntry>) -> io::Result<()> {
        let mut cache = self
            .folder_cache
            .lock()
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "Drive cache lock poisoned"))?;

        cache.insert(folder_id.to_string(), entries);

        Ok(())
    }

    fn remove_cached_folder(&self, folder_id: &str) -> io::Result<()> {
        let mut cache = self
            .folder_cache
            .lock()
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "Drive cache lock poisoned"))?;

        cache.remove(folder_id);

        Ok(())
    }

    async fn create_folder_async(&self, parent_id: &str, name: &str) -> io::Result<DriveEntry> {
        let hub = self.build_hub().await?;

        let folder_metadata = File {
            name: Some(name.to_string()),
            mime_type: Some(DRIVE_FOLDER_MIME_TYPE.to_string()),
            parents: Some(vec![parent_id.to_string()]),
            ..Default::default()
        };

        let empty_content = std::io::Cursor::new(Vec::<u8>::new());

        let (_, created_file) = hub
            .files()
            .create(folder_metadata)
            .param("fields", "id,name,mimeType")
            .add_scope(google_drive3::api::Scope::Full)
            .upload(empty_content, "application/octet-stream".parse().unwrap())
            .await
            .map_err(to_io_error)?;

        file_to_drive_entry(created_file).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::Other,
                "Google Drive did not return created folder metadata",
            )
        })
    }
}

impl DrivePort for GoogleDriveAdapter {
    // sync wrapper required by application layer
    fn list_folder(&self, folder_id: &str) -> io::Result<Vec<DriveEntry>> {
        if let Some(entries) = self.get_cached_folder(folder_id)? {
            return Ok(entries);
        }

        let runtime = tokio::runtime::Runtime::new()?;
        let entries = runtime.block_on(self.list_folder_async(folder_id))?;

        self.store_cached_folder(folder_id, entries.clone())?;

        Ok(entries)
    }

    fn refresh_folder(&self, folder_id: &str) -> io::Result<Vec<DriveEntry>> {
        self.remove_cached_folder(folder_id)?;

        let runtime = tokio::runtime::Runtime::new()?;
        let entries = runtime.block_on(self.list_folder_async(folder_id))?;

        self.store_cached_folder(folder_id, entries.clone())?;

        Ok(entries)
    }

    fn create_folder(&self, parent_id: &str, name: &str) -> io::Result<DriveEntry> {
        let runtime = tokio::runtime::Runtime::new()?;
        let entry = runtime.block_on(self.create_folder_async(parent_id, name))?;

        self.remove_cached_folder(parent_id)?;

        Ok(entry)
    }
}

// converts google file -> domain DriveEntry
fn file_to_drive_entry(file: File) -> Option<DriveEntry> {
    let id = file.id?;
    let name = file.name.unwrap_or_else(|| "(unnamed)".to_string());
    let mime_type = file.mime_type.unwrap_or_default();

    Some(DriveEntry {
        id,
        name,
        is_folder: mime_type == DRIVE_FOLDER_MIME_TYPE,
        mime_type,
    })
}

// converts any error into io::Error
fn to_io_error<E: std::fmt::Display>(error: E) -> io::Error {
    io::Error::new(io::ErrorKind::Other, error.to_string())
}
