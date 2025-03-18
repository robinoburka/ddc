use std::time::Instant;

use anyhow::Result;
use tracing_subscriber::fmt::format::{FmtSpan, Writer};
use tracing_subscriber::fmt::time::FormatTime;

struct SinceStart(Instant);

impl FormatTime for SinceStart {
    fn format_time(&self, w: &mut Writer<'_>) -> std::fmt::Result {
        let elapsed = self.0.elapsed();
        write!(w, "{:.5}s", elapsed.as_secs_f64())
    }
}
#[allow(dead_code)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum LoggingLevel {
    #[default]
    Silent,
    Verbose,
    Traces,
}

impl From<u8> for LoggingLevel {
    fn from(count: u8) -> Self {
        match count {
            0 => LoggingLevel::Silent,
            1 => LoggingLevel::Verbose,
            2.. => LoggingLevel::Traces,
        }
    }
}

pub fn setup_logging(level: LoggingLevel) -> Result<()> {
    let timer = SinceStart(Instant::now());

    let (log_level, tracing_level) = match level {
        LoggingLevel::Silent => (tracing::Level::WARN, FmtSpan::NONE),
        LoggingLevel::Verbose => (tracing::Level::DEBUG, FmtSpan::NONE),
        LoggingLevel::Traces => (tracing::Level::DEBUG, FmtSpan::CLOSE | FmtSpan::ENTER),
    };

    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .compact()
        .with_max_level(log_level)
        .with_thread_names(false)
        .with_thread_ids(false)
        .with_target(true)
        .with_ansi(true)
        .with_timer(timer)
        .with_span_events(tracing_level)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    Ok(())
}
