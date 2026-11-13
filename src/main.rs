mod app;
mod collectors;
mod models;
mod ui;

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut app = app::App::new()?;

    // ratatui::run owns terminal setup and restoration, including on errors.
    ratatui::run(|terminal| app.run(terminal))?;

    Ok(())
}
