use analyzer;

#[derive(PartialEq, Debug)]
pub struct RequestLogAnalyzerResult {
    pub count: usize,
    pub timing: Option<analyzer::TimingResult>,
    pub error: Option<analyzer::aggregated_error_rates::ErrorRatesResult>,
}
