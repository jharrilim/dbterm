use color_eyre::Result;
use dbterm::{app::Runtime, errors, term};

async fn run() -> Result<()> {
    errors::init_hooks()?;
    let terminal = term::init()?;
    // create app and run it
    let runtime = Runtime::new(terminal);

    runtime.run().await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    run().await
}
