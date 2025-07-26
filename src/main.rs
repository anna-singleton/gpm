mod app;
mod config;
mod multi_input;
mod switch_screen;
mod project_item;
mod screen;

use std::io;

use app::App;
use config::Config;

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();

    let mut app = App::default();
    let config: Config = confy::load("gpm", "config").expect("could not load config.");
    let mut forest = config.to_forest();
    app.project_tree = forest;
    let mut app_result = app.run(&mut terminal);
    while *(app_result.as_ref().unwrap_or(&false)) {
        app = App::default();
        forest = config.to_forest();
        app.project_tree = forest;
        app_result = app.run(&mut terminal);
    }

    ratatui::restore();

    match app_result {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}
