#[derive(Clone, Debug)]
pub enum PreviewContent {
    Text {
        title: String,
        content: String,
        truncated: bool,
    },
    Image {
        title: String,
        preview: ImagePreview,
    },
    Unsupported {
        reason: String,
    },
}

#[derive(Clone, Debug)]
pub struct ImagePreview {
    pub width: u32,
    pub height: u32,
    pub cells: Vec<ImagePreviewCell>,
}

#[derive(Clone, Debug)]
pub struct ImagePreviewCell {
    pub character: char,

    pub foreground_red: u8,
    pub foreground_green: u8,
    pub foreground_blue: u8,

    pub background_red: u8,
    pub background_green: u8,
    pub background_blue: u8,
}
