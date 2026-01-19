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

use crate::discovery::{DiscoveryResults, ProgressEvent, ProjectResult, ToolingResult};
use crate::display_tools::{ColorCode, get_size_color_code, get_time_color_code};

#[instrument(level = "debug", skip(out, discovery_results))]
pub fn print_results<W: Write>(out: &mut W, discovery_results: DiscoveryResults) {
    let projects_data: Vec<Record> = discovery_results
        .projects
        .iter()
        .map(Record::from)
        .collect();
    let tooling_data: Vec<ToolingRecord> = discovery_results
        .tools
        .iter()
        .map(ToolingRecord::from)
        .collect();

    let projects_sum: u64 = discovery_results.projects.iter().map(|r| r.size).sum();
    let tooling_sum: u64 = discovery_results.tools.iter().map(|r| r.size).sum();

    let now = SystemTime::now();

    let mut table_tooling_build = Table::new(&tooling_data);
    table_tooling_build.with(Panel::header("Tooling"));
    table_tooling_build.with(Panel::footer(format_size(tooling_sum, DECIMAL)));
    table_tooling_build.with(Modify::new(Rows::last()).with(Color::BOLD));
    table_tooling_build.with(Modify::new(Rows::last()).with(Alignment::right()));
    table_tooling_build.with(Modify::new(Cell::new(0, 0)).with(Color::BOLD));
    table_tooling_build.with(Style::empty());
    tooling_data.iter().enumerate().for_each(|(i, d)| {
        table_tooling_build
            .with(Modify::new(Cell::new(i + 2, 3)).with(time_color_coded(&now, &d.record.time)));
        table_tooling_build
            .with(Modify::new(Cell::new(i + 2, 4)).with(size_color_coded(d.record.size)));
    });
    let table_tooling = table_tooling_build.to_string();
    writeln!(out, "{table_tooling}").expect("Cannot write to stdout");

    let mut table_projects_build = Table::new(&projects_data);
    table_projects_build.with(Panel::header("Projects"));
    table_projects_build.with(Panel::footer(format_size(projects_sum, DECIMAL)));
    table_projects_build.with(Modify::new(Rows::last()).with(Color::BOLD));
    table_projects_build.with(Modify::new(Rows::last()).with(Alignment::right()));
    table_projects_build.with(Modify::new(Cell::new(0, 0)).with(Color::BOLD));
    table_projects_build.with(Style::empty());
    projects_data.iter().enumerate().for_each(|(i, d)| {
        table_projects_build
            .with(Modify::new(Cell::new(i + 2, 2)).with(time_color_coded(&now, &d.time)));
        table_projects_build.with(Modify::new(Cell::new(i + 2, 3)).with(size_color_coded(d.size)));
    });
    let table_projects = table_projects_build.to_string();
    writeln!(out, "{table_projects}").expect("Cannot write to stdout");
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
struct ToolingRecord {
    #[tabled(rename = "Description")]
    description: String,
    #[tabled(inline)]
    record: Record,
}

impl From<&ProjectResult> for Record {
    fn from(value: &ProjectResult) -> Self {
        Self {
            lang: Some(value.lang.to_string()),
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

impl From<&ToolingResult> for ToolingRecord {
    fn from(value: &ToolingResult) -> Self {
        Self {
            description: value.description.clone(),
            record: Record {
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
            },
        }
    }
}

fn size_color_coded(size: u64) -> Color {
    match get_size_color_code(size) {
        ColorCode::None => Color::FG_WHITE,
        ColorCode::Low => Color::FG_GREEN,
        ColorCode::Medium => Color::FG_YELLOW,
        ColorCode::High => Color::FG_RED,
    }
}

fn time_color_coded(now: &SystemTime, time: &Option<SystemTime>) -> Color {
    match get_time_color_code(now, time) {
        ColorCode::None => Color::FG_WHITE,
        ColorCode::Low => Color::FG_GREEN,
        ColorCode::Medium => Color::FG_YELLOW,
        ColorCode::High => Color::FG_RED,
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
