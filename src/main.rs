mod app;
mod screen;
mod project_item;
mod config;

use std::{io, path::PathBuf};

use app::App;
use config::Config;
use project_item::{ProjectItem, ProjectItemType};
use tui_tree_widget::TreeItem;

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let config:Config = confy::load("gpm", "config").expect("could not load config.");
    let forest = config.to_forest();
    let mut app = App::default();
    app.project_tree = forest;
    let app_result = app.run(&mut terminal);
    ratatui::restore();
    app_result
}
