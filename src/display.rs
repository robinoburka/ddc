use std::io::Write;
use std::time::{Duration, SystemTime};

use chrono::{DateTime, Local};
use crossbeam::channel::Receiver;
use humansize::{DECIMAL, format_size};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use tabled::settings::object::Rows;
use tabled::settings::{Alignment, Color, Modify, Panel, Style, object::Cell};
use tabled::{Table, Tabled};
use tracing::instrument;

use crate::discovery::{DiscoveryResult, ProgressEvent, ResultType};

#[instrument(level = "debug", skip(out, discovery_results))]
pub fn print_results<W: Write>(out: &mut W, discovery_results: Vec<DiscoveryResult>) {
    let mut discovery_data = vec![];
    let mut static_data = vec![];

    let mut discovery_sum: u64 = 0;
    let mut static_sum: u64 = 0;

    for result in discovery_results {
        match result.result_type {
            ResultType::Discovery => {
                discovery_sum += result.size;
                discovery_data.push(Record::from(result))
            }
            ResultType::Static(ref description) => {
                static_sum += result.size;
                if result.size != 0 {
                    static_data.push(StaticRecord {
                        description: description.clone(),
                        record: Record::from(result),
                    });
                }
            }
        }
    }

    let now = SystemTime::now();

    let mut table_static_build = Table::new(&static_data);
    table_static_build.with(Panel::header("Tooling"));
    table_static_build.with(Panel::footer(format_size(static_sum, DECIMAL)));
    table_static_build.with(Modify::new(Rows::last()).with(Color::BOLD));
    table_static_build.with(Modify::new(Rows::last()).with(Alignment::right()));
    table_static_build.with(Modify::new(Cell::new(0, 0)).with(Color::BOLD));
    table_static_build.with(Style::empty());
    static_data.iter().enumerate().for_each(|(i, d)| {
        table_static_build
            .with(Modify::new(Cell::new(i + 2, 3)).with(time_color_coded(&now, &d.record.time)));
        table_static_build
            .with(Modify::new(Cell::new(i + 2, 4)).with(size_color_coded(d.record.size)));
    });
    let table_static = table_static_build.to_string();
    writeln!(out, "{table_static}").expect("Cannot write to stdout");

    let mut table_discovery_build = Table::new(&discovery_data);
    table_discovery_build.with(Panel::header("Projects"));
    table_discovery_build.with(Panel::footer(format_size(discovery_sum, DECIMAL)));
    table_discovery_build.with(Modify::new(Rows::last()).with(Color::BOLD));
    table_discovery_build.with(Modify::new(Rows::last()).with(Alignment::right()));
    table_discovery_build.with(Modify::new(Cell::new(0, 0)).with(Color::BOLD));
    table_discovery_build.with(Style::empty());
    discovery_data.iter().enumerate().for_each(|(i, d)| {
        table_discovery_build
            .with(Modify::new(Cell::new(i + 2, 2)).with(time_color_coded(&now, &d.time)));
        table_discovery_build.with(Modify::new(Cell::new(i + 2, 3)).with(size_color_coded(d.size)));
    });
    let table_discovery = table_discovery_build.to_string();
    writeln!(out, "{table_discovery}").expect("Cannot write to stdout");
}

#[derive(Tabled)]
struct Record {
    #[tabled(rename = "Lang", display("tabled::derive::display::option", ""))]
    lang: Option<String>,
    #[tabled(rename = "Path")]
    path: String,
    #[tabled(rename = "Last change", display("tabled::derive::display::option", ""))]
    human_time: Option<String>,
    #[tabled(skip)]
    time: Option<SystemTime>,
    #[tabled(rename = "Size")]
    human_size: String,
    #[tabled(skip)]
    size: u64,
}

#[derive(Tabled)]
struct StaticRecord {
    #[tabled(rename = "Description")]
    description: String,
    #[tabled(inline)]
    record: Record,
}

impl From<DiscoveryResult> for Record {
    fn from(value: DiscoveryResult) -> Self {
        Self {
            lang: value.lang.map(|l| l.to_string()),
            time: value.last_update,
            human_time: value.last_update.map(|t| {
                DateTime::<Local>::from(t)
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string()
            }),
            path: value.path.display().to_string(),
            human_size: format_size(value.size, DECIMAL),
            size: value.size,
        }
    }
}

fn size_color_coded(size: u64) -> Color {
    if size < 1000 * 1000 * 90 {
        Color::FG_GREEN
    } else if size < 1000 * 1000 * 900 {
        Color::FG_YELLOW
    } else {
        Color::FG_RED
    }
}

fn time_color_coded(now: &SystemTime, time: &Option<SystemTime>) -> Color {
    match time {
        None => Color::FG_WHITE, // Wouldn't be displayed anyway
        Some(system_time) => match now.duration_since(*system_time) {
            Err(_) => Color::FG_WHITE, // Future time; shouldn't happen
            Ok(duration) => {
                if duration < Duration::from_days(14) {
                    Color::FG_GREEN
                } else if duration < Duration::from_days(60) {
                    Color::FG_YELLOW
                } else {
                    Color::FG_RED
                }
            }
        },
    }
}

pub fn display_progress_bar(events_receiver: Receiver<ProgressEvent>) {
    let mp = MultiProgress::new();

    let spinner_style =
        ProgressStyle::with_template("{prefix:<12.bold} {spinner:.cyan} {msg}").unwrap();
    let bar_style = ProgressStyle::with_template(
        "{prefix:>12} [{bar:40.cyan/blue}] {human_pos:.dim}/{human_len:.dim} {msg}",
    )
    .unwrap()
    .progress_chars("##-");

    let mut scan_parent: Option<ProgressBar> = None;
    let mut definitions_pb: Option<ProgressBar> = None;
    let mut paths_pb: Option<ProgressBar> = None;
    let mut discover_parent: Option<ProgressBar> = None;
    let mut detectors_pb: Option<ProgressBar> = None;

    for event in events_receiver.iter() {
        match event {
            ProgressEvent::WalkStart { count } => {
                let parent = mp.add(ProgressBar::new_spinner());
                parent.set_prefix("Scan");
                parent.set_style(spinner_style.clone());
                parent.set_message("scanning directories");
                parent.enable_steady_tick(Duration::from_millis(100));
                scan_parent = Some(parent);

                let defs = mp.add(ProgressBar::new(count as u64));
                defs.set_prefix("definitions");
                defs.set_style(bar_style.clone());
                definitions_pb = Some(defs);

                let load = mp.add(ProgressBar::new(0));
                load.set_prefix("paths");
                load.set_style(bar_style.clone());
                paths_pb = Some(load);
            }
            ProgressEvent::WalkAddPaths { count } => {
                if let Some(pb) = &definitions_pb {
                    pb.inc(1);
                }
                if let Some(pb) = &paths_pb {
                    let new_len = pb.length().unwrap_or(0) + count as u64;
                    pb.set_length(new_len);
                }
            }
            ProgressEvent::WalkAdvance => {
                if let Some(pb) = &paths_pb {
                    pb.inc(1);
                }
            }
            ProgressEvent::WalkFinished => {
                if let Some(pb) = &definitions_pb {
                    pb.finish();
                }
                if let Some(pb) = &paths_pb {
                    pb.finish();
                }
                if let Some(pb) = &scan_parent {
                    pb.finish_with_message("done");
                }
            }
            ProgressEvent::DiscoveryStart { count } => {
                let parent = mp.add(ProgressBar::new_spinner());
                parent.set_prefix("Discover");
                parent.set_style(spinner_style.clone());
                parent.set_message("analyzing directories");
                parent.enable_steady_tick(Duration::from_millis(100));
                discover_parent = Some(parent);

                let pb = mp.add(ProgressBar::new(count as u64));
                pb.set_prefix("detectors");
                pb.set_style(bar_style.clone());
                detectors_pb = Some(pb);
            }
            ProgressEvent::DiscoveryAdvance => {
                if let Some(pb) = &detectors_pb {
                    pb.inc(1);
                }
            }
            ProgressEvent::DiscoveryFinished => {
                if let Some(pb) = &detectors_pb {
                    pb.finish();
                }
                if let Some(pb) = &discover_parent {
                    pb.finish_with_message("done");
                }
            }
        }
    }

    let _ = mp.clear();
}

#[cfg(test)]
mod tests {
    use crossbeam::channel;

    use super::*;

    #[test]
    fn test_size_color_coding() {
        assert_eq!(size_color_coded(1000), Color::FG_GREEN);
        assert_eq!(size_color_coded(1000 * 1000), Color::FG_GREEN);
        assert_eq!(size_color_coded(50 * 1000 * 1000), Color::FG_GREEN);
        assert_eq!(size_color_coded(80 * 1000 * 1000), Color::FG_GREEN);
        assert_eq!(size_color_coded(90 * 1000 * 1000), Color::FG_YELLOW);
        assert_eq!(size_color_coded(100 * 1000 * 1000), Color::FG_YELLOW);
        assert_eq!(size_color_coded(500 * 1000 * 1000), Color::FG_YELLOW);
        assert_eq!(size_color_coded(800 * 1000 * 1000), Color::FG_YELLOW);
        assert_eq!(size_color_coded(900 * 1000 * 1000), Color::FG_RED);
        assert_eq!(size_color_coded(1000 * 1000 * 1000), Color::FG_RED);
    }

    #[test]
    fn test_time_color_coding() {
        let now = SystemTime::now();

        assert_eq!(time_color_coded(&now, &None), Color::FG_WHITE);
        assert_eq!(
            time_color_coded(&now, &Some(now - Duration::from_days(1))),
            Color::FG_GREEN
        );
        assert_eq!(
            time_color_coded(&now, &Some(now - Duration::from_days(10))),
            Color::FG_GREEN
        );
        assert_eq!(
            time_color_coded(&now, &Some(now - Duration::from_days(20))),
            Color::FG_YELLOW
        );
        assert_eq!(
            time_color_coded(&now, &Some(now - Duration::from_days(50))),
            Color::FG_YELLOW
        );
        assert_eq!(
            time_color_coded(&now, &Some(now - Duration::from_days(60))),
            Color::FG_RED
        );
        assert_eq!(
            time_color_coded(&now, &Some(now - Duration::from_days(70))),
            Color::FG_RED
        );
    }

    #[test]
    fn test_display_progress_bar_consumes_messages() {
        // This test has lower value. It just tests, that display_progress_bar
        // is able to consume reasonably looking report. It doesn't check the visual reaction though.
        let (tx, rx) = channel::unbounded();

        tx.send(ProgressEvent::WalkStart { count: 1 }).unwrap();
        tx.send(ProgressEvent::WalkAddPaths { count: 1 }).unwrap();
        tx.send(ProgressEvent::WalkAddPaths { count: 1 }).unwrap();
        tx.send(ProgressEvent::WalkAdvance).unwrap();
        tx.send(ProgressEvent::WalkAdvance).unwrap();
        tx.send(ProgressEvent::WalkFinished).unwrap();
        drop(tx);

        let eval_rx = rx.clone();
        display_progress_bar(rx);

        assert_eq!(eval_rx.len(), 0);
    }
}
