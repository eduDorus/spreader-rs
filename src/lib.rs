use anyhow::Result;
use tracing::Level;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::fmt::Subscriber;

pub fn setup_tracing_subscriber() -> Result<()> {
    let subscriber = Subscriber::builder()
        .compact()
        .with_max_level(Level::INFO)
        .with_thread_ids(true)
        .with_target(false)
        .with_span_events(FmtSpan::NONE)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    Ok(())
}
