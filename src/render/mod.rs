pub mod graphite;
pub mod prometheus;

use result;
use timing_analyzer;
use error_analyzer;

pub trait Renderer {
    fn render(&mut self, result: result::RequestLogAnalyzerResult) -> ();
}

pub struct TerminalRenderer {}

impl TerminalRenderer {
    pub fn new() -> TerminalRenderer {
        TerminalRenderer {}
    }
}

impl Renderer for TerminalRenderer {
    fn render(&mut self, result: result::RequestLogAnalyzerResult) -> () {
        println!("count:\t{}", result.count);
        match result.timing {
            Some(timing) => {
                println!("time.avg:\t{}", timing.avg);
                println!("time.min:\t{}", timing.min);
                println!("time.median:\t{}", timing.median);
                println!("time.90percent:\t{}", timing.percentile90);
                println!("time.max:\t{}", timing.max);
            }
            None => warn!("No matching log lines for timing results."),
        }

        match result.error {
            Some(error) => {
                println!("error.client_error_4xx_rate:\t{}", error.client_error_4xx);
                println!("error.server_error_5xx_rate:\t{}", error.server_error_5xx);
            }
            None => warn!("No matching log lines for error rate results."),
        }
    }
}

#[cfg(test)]
mod tests {
    use timing_analyzer;
    use error_analyzer;
    use super::*;

    fn get_result_fixture() -> result::RequestLogAnalyzerResult {
        result::RequestLogAnalyzerResult {
            count: 3,
            timing: Some(timing_analyzer::TimingResult {
                max: 100,
                min: 1,
                avg: 37,
                median: 10,
                percentile90: 100,
            }),
            error: Some(error_analyzer::ErrorRatesResult {
                client_error_4xx: 0.1,
                server_error_5xx: 0.2,
            }),
        }
    }

    #[test]
    fn test_terminal_renderer() {
        let mut renderer = TerminalRenderer::new();

        let result = get_result_fixture();

        renderer.render(result);
    }
}
