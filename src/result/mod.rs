use timing_analyzer;
use error_analyzer;

#[derive(PartialEq, Debug)]
pub struct RequestLogAnalyzerResult {
    pub count: usize,
    pub timing: timing_analyzer::RequestLogAnalyzerResult,
    pub error: error_analyzer::ErrorRatesResult,
}


#[cfg(test)]
mod tests {
    use super::*;
}
