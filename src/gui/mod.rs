mod app;
mod browser;
mod list;
mod table;
mod tabs;

use std::path::Path;
use std::rc::Rc;

use anyhow::Context;

use crate::browse::collect_results;
use crate::cli::{BrowseArgs, UiConfig};
use crate::vcs_postprocess::vcs_postprocess;

pub fn run(cmd_args: &BrowseArgs, ui_config: &UiConfig, home_dir: &Path) -> anyhow::Result<()> {
    let results = collect_results(cmd_args, ui_config, home_dir)?;
    let db = Rc::new(
        results
            .db
            .context("Discovery finished without a files database")?,
    );
    let vcs = vcs_postprocess(&results.projects, results.vcs);
    let projects = results.projects;
    let tooling = results.tools;

    iced::application(
        move || app::App::new(projects.clone(), tooling.clone(), vcs.clone(), db.clone()),
        app::App::update,
        app::App::view,
    )
    .title("ddc")
    .subscription(app::App::subscription)
    .theme(iced::Theme::Dark)
    .run()
    .context("Failed to run the GUI")
}
