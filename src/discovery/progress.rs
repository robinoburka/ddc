#[derive(Debug)]
/// Enum defining protocol of progress reporting.
#[derive(PartialEq)]
pub enum ProgressEvent {
    /// Directory scan has started
    WalkStart { count: usize },
    /// Directory scan has advanced, and adds new paths to load
    WalkAddPaths { count: usize },
    /// Loading paths advanced
    WalkAdvance,
    /// Complete scan has finished
    WalkFinished,
    /// Discovery started, and a count of detectors is provided
    DiscoveryStart { count: usize },
    /// Discovery has advanced, one detector finished
    DiscoveryAdvance,
    /// Discovery has finished
    DiscoveryFinished,
}

pub trait ProgressReporter: Send + Sync + Clone + 'static {
    fn report(&self, event: ProgressEvent);
}
