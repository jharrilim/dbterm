use color_eyre::eyre::Result;
use crossterm::event::Event;

pub trait EventHandler {
    fn handle_event(&mut self, event: Event) -> Result<bool>;
}
