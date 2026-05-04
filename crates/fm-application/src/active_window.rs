// tracks the currently focused window to route global actions (e.g. new dir) to the correct file system (local or drive)
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActiveWindow {
    Explorer(u32),
    Drive(u32),
}
