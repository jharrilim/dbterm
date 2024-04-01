use std::{
    io::{self, Stdout, Write},
    time::Duration,
};

use color_eyre::{eyre::WrapErr, Result};
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;

pub fn init() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode().context("enable raw mode")?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)
        .wrap_err("enter alternate screen")?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

pub fn restore<W: Write>(writer: &mut W) -> Result<()> {
    disable_raw_mode().context("disable raw mode")?;
    execute!(writer, LeaveAlternateScreen)
        .wrap_err("leave alternate screen")?;
    Ok(())
}

pub fn next_event(timeout: Duration) -> Result<Option<Event>> {
    if !event::poll(timeout)? {
        return Ok(None);
    }
    let event = event::read()?;
    Ok(Some(event))
}
