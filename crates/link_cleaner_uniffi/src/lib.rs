#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct CleanReport {
    pub output: String,
    pub urls_found: u64,
    pub urls_modified: u64,
    pub params_removed: u64,
}

#[uniffi::export]
pub fn clean_text(input: &str) -> String {
    link_cleaner_core::clean_text(input)
}

#[uniffi::export]
pub fn clean_text_with_report(input: &str) -> CleanReport {
    let report = link_cleaner_core::clean_text_with_report(input);
    CleanReport {
        output: report.output,
        urls_found: report.urls_found as u64,
        urls_modified: report.urls_modified as u64,
        params_removed: report.params_removed as u64,
    }
}

uniffi::setup_scaffolding!();
