use std::fs;
use std::path::{Path, PathBuf};

use fm_application::FileSystemPort;
use fm_domain::{FileNode, NodeType};

#[derive(Clone, Default)]
pub struct StdFileSystem;

impl FileSystemPort for StdFileSystem {
    fn current_dir(&self) -> PathBuf {
        std::env::current_dir().unwrap_or_else(|_| {
            #[cfg(target_family = "unix")]
            {
                PathBuf::from("/")
            }

            #[cfg(target_os = "windows")]
            {
                PathBuf::from(r"C:\")
            }
        })
    }

    fn roots(&self) -> Vec<FileNode> {
        #[cfg(target_family = "unix")]
        {
            vec![FileNode {
                name: "/".to_string(),
                path: PathBuf::from("/"),
                node_type: NodeType::Root,
            }]
        }

        #[cfg(target_os = "windows")]
        {
            vec![FileNode {
                name: "C:\\".to_string(),
                path: PathBuf::from(r"C:\"),
                node_type: NodeType::Root,
            }]
        }
    }

    fn list_dir(&self, path: &Path) -> Vec<FileNode> {
        let mut items: Vec<FileNode> = match fs::read_dir(path) {
            Ok(entries) => entries
                .flatten()
                .filter_map(|entry| {
                    let file_type = entry.file_type().ok()?;

                    let node_type = if file_type.is_dir() {
                        NodeType::Directory
                    } else if file_type.is_file() {
                        NodeType::File
                    } else {
                        return None;
                    };

                    Some(FileNode {
                        name: entry.file_name().to_string_lossy().to_string(),
                        path: entry.path(),
                        node_type,
                    })
                })
                .collect(),
            Err(_) => Vec::new(),
        };

        items.sort_by(|a, b| {
            use std::cmp::Ordering;

            match (a.node_type, b.node_type) {
                (NodeType::Directory, NodeType::File) => Ordering::Less,
                (NodeType::File, NodeType::Directory) => Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            }
        });

        items
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }
}