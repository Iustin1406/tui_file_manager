use fm_application::FileSystemPort;
use fm_domain::{FileNode, NodeType};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use trash;

#[derive(Clone, Default)]
pub struct StdFileSystem;

impl StdFileSystem {
    fn copy_dir_recursive(source: &Path, destination: &Path) -> io::Result<()> {
        fs::create_dir(destination)?;

        for entry_result in fs::read_dir(source)? {
            let entry = entry_result?;
            let child_source = entry.path();
            let child_destination = destination.join(entry.file_name());

            let metadata = fs::symlink_metadata(&child_source)?;

            if metadata.is_dir() {
                Self::copy_dir_recursive(&child_source, &child_destination)?;
            } else if metadata.is_file() {
                fs::copy(&child_source, &child_destination)?;
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    format!(
                        "Unsupported filesystem entry type: {}",
                        child_source.display()
                    ),
                ));
            }
        }

        Ok(())
    }

    fn is_subpath(path: &Path, potential_parent: &Path) -> bool {
        path.starts_with(potential_parent)
    }

    fn delete_recursively(path: &Path) -> io::Result<()> {
        let metadata = fs::symlink_metadata(path)?;

        if metadata.is_dir() {
            fs::remove_dir_all(path)?;
        } else if metadata.is_file() || metadata.file_type().is_symlink() {
            fs::remove_file(path)?;
        } else {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                format!("Unsupported filesystem entry type: {}", path.display()),
            ));
        }

        Ok(())
    }
}

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

    fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
        std::fs::rename(from, to)
    }

    fn copy_entry(&self, source: &Path, destination_dir: &Path) -> io::Result<PathBuf> {
        if !source.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Source does not exist: {}", source.display()),
            ));
        }

        if !destination_dir.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!(
                    "Destination directory does not exist: {}",
                    destination_dir.display()
                ),
            ));
        }

        if !destination_dir.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "Destination is not a directory: {}",
                    destination_dir.display()
                ),
            ));
        }

        let entry_name = source.file_name().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Source has no final path component: {}", source.display()),
            )
        })?;

        let destination_path = destination_dir.join(entry_name);

        if destination_path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("Destination already exists: {}", destination_path.display()),
            ));
        }

        let metadata = fs::symlink_metadata(source)?;

        if metadata.is_dir() {
            if Self::is_subpath(destination_dir, source) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!(
                        "Cannot copy a directory into itself or one of its descendants: {} -> {}",
                        source.display(),
                        destination_dir.display()
                    ),
                ));
            }

            Self::copy_dir_recursive(source, &destination_path)?;
        } else if metadata.is_file() {
            fs::copy(source, &destination_path)?;
        } else {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                format!("Unsupported filesystem entry type: {}", source.display()),
            ));
        }

        Ok(destination_path)
    }

    fn move_entry(&self, source: &Path, destination_dir: &Path) -> io::Result<PathBuf> {
        if !source.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Source does not exist: {}", source.display()),
            ));
        }

        if !destination_dir.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!(
                    "Destination directory does not exist: {}",
                    destination_dir.display()
                ),
            ));
        }

        if !destination_dir.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "Destination is not a directory: {}",
                    destination_dir.display()
                ),
            ));
        }

        let entry_name = source.file_name().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Source has no final path component: {}", source.display()),
            )
        })?;

        let destination_path = destination_dir.join(entry_name);

        if destination_path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("Destination already exists: {}", destination_path.display()),
            ));
        }

        if source == destination_path {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Source and destination are identical: {}", source.display()),
            ));
        }

        let metadata = fs::symlink_metadata(source)?;

        if metadata.is_dir() && Self::is_subpath(destination_dir, source) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "Cannot move a directory into itself or one of its descendants: {} -> {}",
                    source.display(),
                    destination_dir.display()
                ),
            ));
        }

        fs::rename(source, &destination_path)?;

        Ok(destination_path)
    }

    fn move_to_trash(&self, path: &Path) -> io::Result<()> {
        if !path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Path does not exist: {}", path.display()),
            ));
        }

        trash::delete(path).map_err(|err| io::Error::new(io::ErrorKind::Other, err.to_string()))
    }
    fn delete_permanently(&self, path: &Path) -> io::Result<()> {
        if !path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Path does not exist: {}", path.display()),
            ));
        }

        Self::delete_recursively(path)
    }
    fn create_dir(&self, parent_dir: &Path, name: &str) -> io::Result<PathBuf> {
        if !parent_dir.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Parent directory does not exist: {}", parent_dir.display()),
            ));
        }

        if !parent_dir.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Parent path is not a directory: {}", parent_dir.display()),
            ));
        }

        let new_dir_path = parent_dir.join(name);

        if new_dir_path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("Directory already exists: {}", new_dir_path.display()),
            ));
        }

        fs::create_dir(&new_dir_path)?;

        Ok(new_dir_path)
    }
}
