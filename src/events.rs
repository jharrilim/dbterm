use color_eyre::eyre::Result;
use crossterm::event::Event;
use tokio::sync::mpsc::UnboundedSender;

use crate::data::{AppCommand, Ctx};

pub trait EventHandler {
    fn handle_event(
        &mut self,
        event: Event,
        ctx: &Ctx,
        tx: &UnboundedSender<AppCommand>,
    ) -> Result<bool>;
}
