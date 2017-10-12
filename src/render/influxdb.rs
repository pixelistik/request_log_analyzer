use render::Renderer;
use result;
use hyper::client;

pub struct InfluxDbRenderer {
    write_url: String,
}

impl InfluxDbRenderer {
    pub fn new(write_url: String)
               -> InfluxDbRenderer {
        InfluxDbRenderer {
            write_url: write_url,
        }
    }
}

impl Renderer for InfluxDbRenderer {
    fn render(&mut self, result: result::RequestLogAnalyzerResult) -> () {		
		let client = client::Client::new();
		let res = client.post(&self.write_url)
					.body(&post_body(result))
					.send()
					.unwrap();
    }
}

fn post_body(result: result::RequestLogAnalyzerResult) -> String {
	let mut timing_values = String::from("");
	let mut error_rate_values = String::from("");
	
	match result.timing {
		Some(timing) => {
			timing_values = format!("\
				time_max={} \
				time_min={} \
				time_avg={} \
				time_median={} \
				time_90percent={}", 
				timing.max,
				timing.min,
				timing.avg,
				timing.median,
				timing.percentile90);
		}
		None => warn!("No matching log lines in file."),
	}

	match result.error {
		Some(error) => {
			error_rate_values = format!("\
				client_error_4xx_rate={} \
				server_error_5xx_rate={}",
				error.client_error_4xx,
				error.server_error_5xx);
		}
		None => warn!("No matching log lines in file."),
	}
	
	format!("request_log count={} {} {}", result.count, timing_values, error_rate_values)
}

#[cfg(test)]
mod tests {
    use std::str;
    use analyzer;
    use super::*;

    fn get_result_fixture() -> result::RequestLogAnalyzerResult {
        result::RequestLogAnalyzerResult {
            count: 3,
            timing: Some(analyzer::TimingResult {
                max: 100,
                min: 1,
                avg: 37,
                median: 10,
                percentile90: 100,
                count: 3,
            }),
            error: Some(analyzer::aggregated_error_rates::ErrorRatesResult {
                client_error_4xx: 0.1,
                server_error_5xx: 0.2,
            }),
        }
    }
	
	#[test]
	fn test_instantiate() {
		InfluxDbRenderer::new(String::from("http://example.com/write?db=testdb"));
	}
	
	#[test]
	fn test_post_body() {
		let result = post_body(get_result_fixture());
		
		assert!(result.starts_with("request_log "));
		
		assert!(result.contains("count=3"));
		assert!(result.contains("time_max=100"));
		assert!(result.contains("time_min=1"));
		assert!(result.contains("time_avg=37"));
		assert!(result.contains("time_median=10"));
		assert!(result.contains("time_90percent=100"));
		assert!(result.contains("client_error_4xx_rate=0.1"));
		assert!(result.contains("server_error_5xx_rate=0.2"));
		
	}
	
	#[test]
	fn test_post_body_empty() {
		let result = post_body(result::RequestLogAnalyzerResult {
            count: 0,
            timing: None,
            error: None,
        });
		
		assert!(result.starts_with("request_log "));
		
		assert!(result.contains("count=0"));
		
		// Don't include empty fields
		assert!(!result.contains("time_max="));
		assert!(!result.contains("time_min="));
		assert!(!result.contains("time_avg="));
		assert!(!result.contains("time_median="));
		assert!(!result.contains("time_90percent="));
		assert!(!result.contains("client_error_4xx_rate="));
		assert!(!result.contains("server_error_5xx_rate="));
	}
}
