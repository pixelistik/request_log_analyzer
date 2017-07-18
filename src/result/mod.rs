use timing_analyzer;
use error_analyzer;

#[derive(PartialEq, Debug)]
pub struct RequestLogAnalyzerResult {
    pub count: usize,
    pub timing: Option<timing_analyzer::RequestLogAnalyzerResult>,
    pub error: Option<error_analyzer::ErrorRatesResult>,
}


#[cfg(test)]
mod tests {
    use super::*;
}
