use color_eyre::Result;
use dbterm::IoEvent;
use dbterm::{app::Runtime, errors, term};
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::Mutex;

async fn run() -> Result<()> {
    errors::init_hooks()?;
    let terminal = term::init()?;
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<IoEvent>();
    // create app and run it
    let app = Arc::new(Mutex::new(Runtime::new(tx, terminal)));

    let mut cloned_app = Arc::clone(&app);
    tokio::spawn(async move {
        network_handler(rx, &mut cloned_app).await;
    });

    app.lock().await.run().await?;

    Ok(())
}

async fn network_handler(io_rx: UnboundedReceiver<IoEvent>, app: &mut Arc<Mutex<Runtime>>) {}

#[tokio::main]
async fn main() -> Result<()> {
    run().await
}
