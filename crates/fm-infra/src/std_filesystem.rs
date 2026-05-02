use fm_application::FileSystemPort;
use fm_domain::{
    EntryProperties, FileNode, ImagePreview, ImagePreviewCell, NodeType, PreviewContent,
};
use image::GenericImageView;
use std::fs;
use std::io;
use std::io::Read;
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

    fn calculate_entry_size(path: &Path) -> io::Result<u64> {
        let metadata = fs::symlink_metadata(path)?;

        if metadata.is_file() {
            return Ok(metadata.len());
        }

        if metadata.is_dir() {
            let mut total_size = 0u64;

            for entry_result in fs::read_dir(path)? {
                let entry = entry_result?;
                let child_path = entry.path();

                match Self::calculate_entry_size(&child_path) {
                    Ok(child_size) => {
                        total_size += child_size;
                    }
                    Err(_) => {
                        // if we can't access the child entry, we skip it and continue with the rest
                        continue;
                    }
                }
            }

            return Ok(total_size);
        }

        Ok(0)
    }

    fn reduce_color(value: u8) -> u8 {
        (value / 32) * 32
    }

    fn try_build_image_preview(
        path: &Path,
        max_width: u32,
        max_height: u32,
    ) -> io::Result<Option<PreviewContent>> {
        let image = match image::open(path) {
            Ok(image) => image,
            Err(_) => return Ok(None),
        };

        let image = image.adjust_contrast(25.0).brighten(5);

        let title = path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());

        let preview = Self::build_image_preview(image, max_width, max_height)?;

        Ok(Some(PreviewContent::Image { title, preview }))
    }

    fn build_image_preview(
        image: image::DynamicImage,
        max_width: u32,
        max_height: u32,
    ) -> io::Result<ImagePreview> {
        let (original_width, original_height) = image.dimensions();

        if original_width == 0 || original_height == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Image has invalid dimensions",
            ));
        }

        let (resized_width, resized_pixel_height) = Self::calculate_preview_pixel_dimensions(
            original_width,
            original_height,
            max_width,
            max_height,
        );

        let resized = image.resize_exact(
            resized_width,
            resized_pixel_height,
            image::imageops::FilterType::Lanczos3,
        );

        let rgba = resized.to_rgba8();

        let preview_width = resized_width;
        let preview_height = resized_pixel_height.div_ceil(2);

        let mut cells = Vec::with_capacity((preview_width * preview_height) as usize);

        for cell_y in 0..preview_height {
            let top_y = cell_y * 2;
            let bottom_y = top_y + 1;

            for x in 0..preview_width {
                let top = rgba.get_pixel(x, top_y);
                let [top_r, top_g, top_b, top_a] = top.0;

                let (foreground_red, foreground_green, foreground_blue) = if top_a == 0 {
                    (0, 0, 0)
                } else {
                    (
                        Self::reduce_color(top_r),
                        Self::reduce_color(top_g),
                        Self::reduce_color(top_b),
                    )
                };

                let (background_red, background_green, background_blue) =
                    if bottom_y < resized_pixel_height {
                        let bottom = rgba.get_pixel(x, bottom_y);
                        let [bottom_r, bottom_g, bottom_b, bottom_a] = bottom.0;

                        if bottom_a == 0 {
                            (0, 0, 0)
                        } else {
                            (
                                Self::reduce_color(bottom_r),
                                Self::reduce_color(bottom_g),
                                Self::reduce_color(bottom_b),
                            )
                        }
                    } else {
                        (0, 0, 0)
                    };

                cells.push(ImagePreviewCell {
                    character: '▀',

                    foreground_red,
                    foreground_green,
                    foreground_blue,

                    background_red,
                    background_green,
                    background_blue,
                });
            }
        }

        Ok(ImagePreview {
            width: preview_width,
            height: preview_height,
            cells,
        })
    }

    fn calculate_preview_pixel_dimensions(
        original_width: u32,
        original_height: u32,
        max_cell_width: u32,
        max_cell_height: u32,
    ) -> (u32, u32) {
        let max_cell_width = max_cell_width.max(1);
        let max_cell_height = max_cell_height.max(1);

        let max_pixel_width = max_cell_width;
        let max_pixel_height = max_cell_height * 2;

        let width_scale = max_pixel_width as f64 / original_width as f64;
        let height_scale = max_pixel_height as f64 / original_height as f64;

        let scale = width_scale.min(height_scale).min(1.0);

        let resized_width = ((original_width as f64 * scale).round() as u32).max(1);
        let resized_height = ((original_height as f64 * scale).round() as u32).max(1);

        (resized_width, resized_height)
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
                size: None,
                modified: None,
                is_hidden: false,
            }]
        }

        #[cfg(target_os = "windows")]
        {
            vec![FileNode {
                name: "C:\\".to_string(),
                path: PathBuf::from(r"C:\"),
                node_type: NodeType::Root,
                size: None,
                modified: None,
                is_hidden: false,
            }]
        }
    }

    fn list_dir(&self, path: &Path) -> Vec<FileNode> {
        match fs::read_dir(path) {
            Ok(entries) => entries
                .flatten()
                .filter_map(|entry| {
                    let file_type = entry.file_type().ok()?;
                    let metadata = entry.metadata().ok();

                    let node_type = if file_type.is_dir() {
                        NodeType::Directory
                    } else if file_type.is_file() {
                        NodeType::File
                    } else {
                        return None;
                    };

                    let name = entry.file_name().to_string_lossy().to_string();
                    let size = if file_type.is_file() {
                        metadata.as_ref().map(|m| m.len())
                    } else {
                        None
                    };

                    let modified = metadata.as_ref().and_then(|m| m.modified().ok());
                    let is_hidden = name.starts_with('.');

                    Some(FileNode {
                        name,
                        path: entry.path(),
                        node_type,
                        size,
                        modified,
                        is_hidden,
                    })
                })
                .collect(),
            Err(_) => Vec::new(),
        }
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

    fn get_entry_properties(&self, path: &Path) -> io::Result<EntryProperties> {
        if !path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Path does not exist: {}", path.display()),
            ));
        }

        let metadata = fs::symlink_metadata(path)?;

        let node_type = if metadata.is_dir() {
            NodeType::Directory
        } else if metadata.is_file() {
            NodeType::File
        } else {
            NodeType::Root
        };

        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .filter(|n| !n.is_empty())
            .unwrap_or_else(|| path.to_string_lossy().to_string());

        let size = Some(Self::calculate_entry_size(path)?);

        let modified = metadata.modified().ok();

        let is_hidden = path
            .file_name()
            .map(|n| n.to_string_lossy().starts_with('.'))
            .unwrap_or(false);

        Ok(EntryProperties {
            name,
            path: path.to_path_buf(),
            node_type,
            size,
            modified,
            is_hidden,
        })
    }

    fn open_file(&self, path: &Path) -> io::Result<()> {
        if !path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Path does not exist: {}", path.display()),
            ));
        }

        if !path.is_file() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Path is not a file: {}", path.display()),
            ));
        }

        open::that(path).map_err(|err| io::Error::new(io::ErrorKind::Other, err.to_string()))
    }

    fn preview_entry(
        &self,
        path: &Path,
        max_text_bytes: usize,
        max_image_width: u32,
        max_image_height: u32,
    ) -> io::Result<PreviewContent> {
        if !path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Path does not exist: {}", path.display()),
            ));
        }

        if !path.is_file() {
            return Ok(PreviewContent::Unsupported {
                reason: "Preview is only available for files".to_string(),
            });
        }

        if let Some(image_preview) =
            Self::try_build_image_preview(path, max_image_width, max_image_height)?
        {
            return Ok(image_preview);
        }

        let mut file = fs::File::open(path)?;
        let metadata = file.metadata()?;
        let truncated = metadata.len() > max_text_bytes as u64;

        let mut buffer = vec![0u8; max_text_bytes];
        let bytes_read = file.read(&mut buffer)?;
        buffer.truncate(bytes_read);

        let content = match String::from_utf8(buffer) {
            Ok(text) => text.replace("\r\n", "\n").replace('\t', "    "),
            Err(_) => {
                return Ok(PreviewContent::Unsupported {
                    reason: "Preview is supported only for text files and images".to_string(),
                });
            }
        };

        let title = path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());

        Ok(PreviewContent::Text {
            title,
            content,
            truncated,
        })
    }
}
