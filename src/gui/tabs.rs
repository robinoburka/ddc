//! Column definitions for each tab, plus how a record plugs into the table.

use std::collections::HashSet;
use std::path::Path;
use std::time::SystemTime;

use iced::Length;
use iced::widget::{Text, text};

use super::table::{Column, Record, SortBy, Table, dimmed_size_text, last_update_text, size_text};
use crate::discovery::{ProjectResult, ToolingResult};
use crate::vcs_postprocess::EnrichedVcsResult;

const ICON: Length = Length::Fixed(40.0);
const PATH: Length = Length::FillPortion(6);
const SIZE: Length = Length::Fixed(100.0);
const DATE: Length = Length::Fixed(180.0);

// Projects

impl Record for ProjectResult {
    fn path(&self) -> &Path {
        &self.path
    }
    fn size(&self) -> u64 {
        self.size
    }
    fn last_update(&self) -> Option<SystemTime> {
        self.last_update
    }
}

pub fn projects(results: Vec<ProjectResult>) -> Table<ProjectResult> {
    fn lang(r: &ProjectResult, _: SystemTime) -> Text<'_> {
        text(r.lang.to_string())
    }
    fn path(r: &ProjectResult, _: SystemTime) -> Text<'_> {
        text(r.path.display().to_string())
    }
    fn size(r: &ProjectResult, _: SystemTime) -> Text<'_> {
        size_text(r.size)
    }
    fn updated(r: &ProjectResult, now: SystemTime) -> Text<'_> {
        last_update_text(now, r.last_update)
    }
    fn parent(r: &ProjectResult, _: SystemTime) -> Text<'_> {
        dimmed_size_text(r.parent.as_ref().map(|p| p.size).unwrap_or_default())
    }

    Table::new(
        vec![
            col("", ICON, None, lang),
            col("Project", PATH, Some(SortBy::Project), path),
            col("Size", SIZE, Some(SortBy::Size), size),
            col("Last update", DATE, Some(SortBy::LastUpdate), updated),
            col("Parent size", SIZE, None, parent),
        ],
        &[SortBy::Project, SortBy::Size, SortBy::LastUpdate],
        "projects",
        results,
    )
}

// Tooling

impl Record for ToolingResult {
    fn path(&self) -> &Path {
        &self.path
    }
    fn size(&self) -> u64 {
        self.size
    }
    fn last_update(&self) -> Option<SystemTime> {
        self.last_update
    }
}

pub fn tooling(results: Vec<ToolingResult>) -> Table<ToolingResult> {
    fn lang(r: &ToolingResult, _: SystemTime) -> Text<'_> {
        text(r.lang.to_string())
    }
    fn tool(r: &ToolingResult, _: SystemTime) -> Text<'_> {
        text(format!("{} ({})", r.description, r.path.display()))
    }
    fn size(r: &ToolingResult, _: SystemTime) -> Text<'_> {
        size_text(r.size)
    }
    fn updated(r: &ToolingResult, now: SystemTime) -> Text<'_> {
        last_update_text(now, r.last_update)
    }
    fn info(r: &ToolingResult, _: SystemTime) -> Text<'_> {
        text(r.info.map(|_| "📖 »").unwrap_or_default())
    }

    Table::new(
        vec![
            col("", ICON, None, lang),
            col("Tool", PATH, None, tool),
            col("Size", SIZE, Some(SortBy::Size), size),
            col("Last update", DATE, Some(SortBy::LastUpdate), updated),
            col("Info", Length::Fixed(60.0), None, info),
        ],
        &[SortBy::Size, SortBy::LastUpdate],
        "tools",
        results,
    )
}

// Version controlled

impl Record for EnrichedVcsResult {
    fn path(&self) -> &Path {
        &self.path
    }
    fn size(&self) -> u64 {
        self.size
    }
    fn last_update(&self) -> Option<SystemTime> {
        self.last_update
    }
    fn detected_projects(&self) -> usize {
        self.matched_projects.len()
    }
}

pub fn vcs(results: Vec<EnrichedVcsResult>) -> Table<EnrichedVcsResult> {
    fn detected_size(r: &EnrichedVcsResult) -> u64 {
        r.matched_projects.iter().map(|p| p.size).sum()
    }
    fn path(r: &EnrichedVcsResult, _: SystemTime) -> Text<'_> {
        text(r.path.display().to_string())
    }
    fn size(r: &EnrichedVcsResult, _: SystemTime) -> Text<'_> {
        size_text(r.size)
    }
    fn updated(r: &EnrichedVcsResult, now: SystemTime) -> Text<'_> {
        last_update_text(now, r.last_update)
    }
    fn files(r: &EnrichedVcsResult, _: SystemTime) -> Text<'_> {
        size_text(
            r.size
                .saturating_sub(r.vcs_size)
                .saturating_sub(detected_size(r)),
        )
    }
    fn vcs_dir(r: &EnrichedVcsResult, _: SystemTime) -> Text<'_> {
        dimmed_size_text(r.vcs_size)
    }
    fn dev_dirs(r: &EnrichedVcsResult, _: SystemTime) -> Text<'_> {
        if r.matched_projects.is_empty() {
            text("")
        } else {
            size_text(detected_size(r))
        }
    }
    fn projects(r: &EnrichedVcsResult, _: SystemTime) -> Text<'_> {
        text(detected_projects_summary(r))
    }

    Table::new(
        vec![
            col("", ICON, None, |_, _| text("")),
            col("Path", PATH, Some(SortBy::Project), path),
            col("Size", SIZE, Some(SortBy::Size), size),
            col("Last update", DATE, Some(SortBy::LastUpdate), updated),
            col("Files", Length::Fixed(90.0), None, files),
            col("VCS dir", Length::Fixed(90.0), None, vcs_dir),
            col("Dev dirs", Length::Fixed(90.0), None, dev_dirs),
            col(
                "Projects",
                Length::Fixed(110.0),
                Some(SortBy::DetectedProjects),
                projects,
            ),
        ],
        &[
            SortBy::Project,
            SortBy::Size,
            SortBy::LastUpdate,
            SortBy::DetectedProjects,
        ],
        "repositories",
        results,
    )
}

fn detected_projects_summary(result: &EnrichedVcsResult) -> String {
    let count = result.matched_projects.len();
    if count == 0 {
        return String::new();
    }

    let mut seen = HashSet::new();
    let icons: Vec<_> = result
        .matched_projects
        .iter()
        .map(|p| p.lang.to_string())
        .filter(|icon| seen.insert(icon.clone()))
        .collect();

    format!("{count}: {}", icons.join(" "))
}

fn col<R>(
    title: &'static str,
    width: Length,
    sort: Option<SortBy>,
    cell: for<'a> fn(&'a R, SystemTime) -> Text<'a>,
) -> Column<R> {
    Column {
        title,
        width,
        sort,
        cell,
    }
}
