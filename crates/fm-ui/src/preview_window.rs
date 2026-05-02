use appcui::prelude::*;
use fm_domain::ImagePreview;

#[Window]
pub struct PreviewWindow {
    canvas: Handle<Canvas>,
}

impl PreviewWindow {
    pub fn new(title: &str, content: &str, truncated: bool) -> Self {
        let mut window = Self {
            base: Window::new(
                &format!("Preview - {}", title),
                layout!("a:c,w:80%,h:70%"),
                window::Flags::Sizeable,
            ),
            canvas: Handle::None,
        };

        let line_count = (content.lines().count().max(1) as u32 + 2).max(40);
        let max_line_width = content
            .lines()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(120)
            .max(140) as u32;

        let mut canvas = Canvas::new(
            Size::new(max_line_width, line_count),
            layout!("d:f"),
            canvas::Flags::ScrollBars,
        );

        {
            let surface = canvas.drawing_surface_mut();

            // Clear the surface with spaces to ensure the background color is applied to the entire area
            surface.clear(Character::with_attributes(
                ' ',
                CharAttribute::with_color(Color::White, Color::Black),
            ));

            surface.write_string(
                0,
                0,
                content,
                CharAttribute::with_color(Color::White, Color::Black),
                true,
            );

            if truncated {
                surface.write_string(
                    0,
                    line_count as i32 - 1,
                    "--- Preview truncated to 128 KB ---",
                    CharAttribute::with_color(Color::Yellow, Color::Black),
                    true,
                );
            }
        }

        window.canvas = window.add(canvas);

        window
    }

    pub fn new_image(title: &str, preview: &ImagePreview) -> Self {
        let mut window = Self {
            base: Window::new(
                &format!("Preview - {}", title),
                layout!("a:c,w:90%,h:80%"),
                window::Flags::Sizeable,
            ),
            canvas: Handle::None,
        };

        let padding_x = 2;
        let padding_y = 1;

        let canvas_width = preview.width + padding_x * 2;
        let canvas_height = preview.height + padding_y * 2;

        let mut canvas = Canvas::new(
            Size::new(canvas_width.max(1), canvas_height.max(1)),
            layout!("d:f"),
            canvas::Flags::ScrollBars,
        );

        {
            let surface = canvas.drawing_surface_mut();

            // Clear the surface with spaces to ensure the background color is applied to the entire area
            surface.clear(Character::with_attributes(
                ' ',
                CharAttribute::with_color(Color::White, Color::Black),
            ));

            for y in 0..preview.height {
                for x in 0..preview.width {
                    let index = (y * preview.width + x) as usize;

                    let Some(cell) = preview.cells.get(index) else {
                        continue;
                    };

                    let foreground = Color::from_rgb(
                        cell.foreground_red,
                        cell.foreground_green,
                        cell.foreground_blue,
                    );

                    let background = Color::from_rgb(
                        cell.background_red,
                        cell.background_green,
                        cell.background_blue,
                    );

                    surface.write_string(
                        (x + padding_x) as i32,
                        (y + padding_y) as i32,
                        &cell.character.to_string(),
                        CharAttribute::with_color(foreground, background),
                        false,
                    );
                }
            }
        }

        window.canvas = window.add(canvas);

        window
    }
}
