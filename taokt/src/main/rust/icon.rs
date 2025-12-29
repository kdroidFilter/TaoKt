use std::path::Path;

use crate::TaoError;

#[derive(uniffi::Object)]
pub struct Icon {
    pub(crate) inner: tao::window::Icon,
}

#[uniffi::export]
impl Icon {
    #[uniffi::constructor]
    pub fn from_rgba(rgba: Vec<u8>, width: u32, height: u32) -> Result<Self, TaoError> {
        Ok(Self {
            inner: tao::window::Icon::from_rgba(rgba, width, height)?,
        })
    }

    /// Load an image from disk and create a window icon from it.
    ///
    /// On Linux, pass the icon in the resolution it was naturally drawn.
    #[uniffi::constructor]
    pub fn from_file(path: String) -> Result<Self, TaoError> {
        let (rgba, width, height) = load_rgba(path.as_ref())?;
        Self::from_rgba(rgba, width, height)
    }
}

fn load_rgba(path: &Path) -> Result<(Vec<u8>, u32, u32), TaoError> {
    let image = image::open(path)
        .map_err(|e| TaoError::message(format!("Failed to open icon path: {e}")))?
        .into_rgba8();
    let (width, height) = image.dimensions();
    Ok((image.into_raw(), width, height))
}

