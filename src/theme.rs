use ratatui::style::Style;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct Theme {
    pub popup: Style,
}
// Todo: Add theme support
#[allow(unused)]
pub(crate) fn default_theme() -> Theme {
    Theme {
        popup: Style::default(),
    }
}
