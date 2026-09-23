use jian_widgets::ImageDrawMode;

/// Image placement mode carried by a scene node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SceneImageFit {
    #[default]
    Fill,
    Fit,
    Crop,
    Tile,
    Stretch,
    CssRepeat,
}

impl SceneImageFit {
    pub fn to_draw_mode(self) -> ImageDrawMode {
        match self {
            Self::Fill => ImageDrawMode::Fill,
            Self::Fit => ImageDrawMode::Fit,
            Self::Crop => ImageDrawMode::Crop,
            Self::Tile => ImageDrawMode::Tile,
            Self::Stretch => ImageDrawMode::Stretch,
            Self::CssRepeat => ImageDrawMode::CssRepeat,
        }
    }
}
