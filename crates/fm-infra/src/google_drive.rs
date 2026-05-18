use std::collections::HashMap;
use std::io;
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
use std::fs::File as StdFile;
use std::path::{Path, PathBuf};

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

    async fn rename_item_async(&self, file_id: &str, new_name: &str) -> io::Result<DriveEntry> {
        let hub = self.build_hub().await?;

        // only metadata is changed (the item name)
        let metadata = File {
            name: Some(new_name.to_string()),
            ..Default::default()
        };

        // google-drive3 requires update() to go through upload flow,
        // so we send an empty body to keep the existing content unchanged
        let empty_content = std::io::Cursor::new(Vec::<u8>::new());

        let mime_type = "application/octet-stream".parse().map_err(to_io_error)?;

        let (_, updated_file) = hub
            .files()
            .update(metadata, file_id)
            .param("fields", "id,name,mimeType")
            .add_scope(google_drive3::api::Scope::Full)
            .upload(empty_content, mime_type)
            .await
            .map_err(to_io_error)?;

        file_to_drive_entry(updated_file).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::Other,
                "Google Drive did not return renamed item metadata",
            )
        })
    }

    async fn upload_file_async(
        &self,
        local_path: &Path,
        parent_id: &str,
    ) -> io::Result<DriveEntry> {
        let hub = self.build_hub().await?;

        let file_name = local_path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "Invalid local file name")
            })?;

        let metadata = File {
            name: Some(file_name),
            parents: Some(vec![parent_id.to_string()]),
            ..Default::default()
        };

        let file = StdFile::open(local_path)?;

        let mime_type = "application/octet-stream".parse().map_err(to_io_error)?;

        let (_, uploaded_file) = hub
            .files()
            .create(metadata)
            .param("fields", "id,name,mimeType")
            .add_scope(google_drive3::api::Scope::Full)
            .upload(file, mime_type)
            .await
            .map_err(to_io_error)?;

        file_to_drive_entry(uploaded_file).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::Other,
                "Google Drive did not return uploaded file metadata",
            )
        })
    }

    fn upload_folder_recursive(
        &self,
        runtime: &tokio::runtime::Runtime,
        local_folder: &Path,
        parent_id: &str,
    ) -> io::Result<DriveEntry> {
        let folder_name = local_folder
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid folder name"))?;

        let created_folder = runtime.block_on(self.create_folder_async(parent_id, &folder_name))?;

        for entry in std::fs::read_dir(local_folder)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                runtime.block_on(self.upload_file_async(&path, &created_folder.id))?;
            } else if path.is_dir() {
                self.upload_folder_recursive(runtime, &path, &created_folder.id)?;
            }
        }

        Ok(created_folder)
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

    fn rename_item(&self, file_id: &str, new_name: &str) -> io::Result<DriveEntry> {
        let runtime = tokio::runtime::Runtime::new()?;
        let entry = runtime.block_on(self.rename_item_async(file_id, new_name))?;

        let mut cache = self
            .folder_cache
            .lock()
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "Drive cache lock poisoned"))?;

        // Invalidate all cached folders containing this item, since we don't track parent-child relationships in cache
        cache.clear();

        Ok(entry)
    }

    fn upload_file(&self, local_path: &Path, parent_id: &str) -> io::Result<DriveEntry> {
        let runtime = tokio::runtime::Runtime::new()?;
        let entry = runtime.block_on(self.upload_file_async(local_path, parent_id))?;

        self.remove_cached_folder(parent_id)?;

        Ok(entry)
    }

    fn upload_folder(&self, local_folder: &Path, parent_id: &str) -> io::Result<DriveEntry> {
        let runtime = tokio::runtime::Runtime::new()?;

        let entry = self.upload_folder_recursive(&runtime, local_folder, parent_id)?;

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
