//! Graphical interface for the QCY Melobuds Pro.
//!
//! The window is intentionally close to the reference layout: the headphones
//! illustration lives on the left, while the controls are grouped in tabs on
//! the right. Every action still goes through the same protocol and BLE layer
//! used by the CLI.

#[path = "gui/app.rs"]
mod app;
#[path = "gui/ble_task.rs"]
mod ble_task;
#[path = "gui/cli.rs"]
mod cli;
#[path = "gui/device_panel.rs"]
mod device_panel;
#[path = "gui/options.rs"]
mod options;
#[path = "gui/tabs/mod.rs"]
mod tabs;
#[path = "gui/theme.rs"]
mod theme;
#[path = "gui/view.rs"]
mod view;
#[path = "gui/widgets.rs"]
mod widgets;

#[cfg(test)]
#[path = "gui/tests/app.rs"]
mod app_tests;
#[cfg(test)]
#[path = "gui/tests/cli.rs"]
mod cli_tests;

use clap::Parser;
use iced::Size;

fn main() -> iced::Result {
    let cli = cli::Cli::parse();

    iced::application("MelobudsNext - QCY Melobuds Pro", app::update, view::view)
        .theme(theme::theme)
        .window(iced::window::Settings {
            size: Size::new(1200.0, 800.0),
            resizable: false,
            ..Default::default()
        })
        .run_with(move || app::App::new(cli))
}
