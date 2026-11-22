//! Content of each tab shown on the right panel.

use iced::Element;

use crate::app::{App, Message};
use crate::options::Tab;

mod anc;
mod audio;
mod status;
mod touch;
mod wear;

pub(crate) fn content(app: &App) -> Element<'_, Message> {
    match app.tab {
        Tab::Status => status::status_tab(app),
        Tab::Anc => anc::anc_tab(app),
        Tab::Audio => audio::audio_tab(app),
        Tab::Touch => touch::touch_tab(app),
        Tab::Wear => wear::wear_tab(app),
    }
}
