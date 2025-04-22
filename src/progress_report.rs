use indicatif::{ProgressBar, ProgressStyle};

pub struct ProgressReport {
    pb: ProgressBar,
}

impl ProgressReport {
    pub fn new() -> Self {
        let pb = ProgressBar::new(0);
        pb.set_style(ProgressStyle::default_bar()
            .template("{msg} [{elapsed_time}] {bar:.40.white} {pos}/{len}")
            .expect("Failed to set ProgressBar style.")
        );
        pb.set_message("Preparing directory scan");
        Self { pb }
    }

    pub fn start_scan(&self, total: usize) {
        self.pb.set_message("Scanning directories");
        self.pb.set_length(total as u64);
    }

    pub fn tick(&self) {
        self.pb.inc(1);
    }

    pub fn start_analyze(&self, total: usize) {
        self.pb.set_message("Analyzing files");
        self.pb.set_length(total as u64);
    }

    pub fn done(&self) {
        self.pb.finish_with_message("All done");
    }
}