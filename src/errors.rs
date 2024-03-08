use color_eyre::{config::HookBuilder, Result};
use crossterm::execute;

use crate::term;

pub fn init_hooks() -> Result<()> {
    let (panic, error) = HookBuilder::default().into_hooks();
    let panic = panic.into_panic_hook();
    let error = error.into_eyre_hook();
    color_eyre::eyre::set_hook(Box::new(move |e| {
        error(e)
    }))?;
    std::panic::set_hook(Box::new(move |info| {
        let mut stdout = std::io::stdout();
        let _ = term::restore(&mut stdout);
        let _ = execute!(stdout, crossterm::cursor::Show);
        panic(info)
    }));
    Ok(())
}
