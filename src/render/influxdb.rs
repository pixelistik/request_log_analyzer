use chrono::*;
use std::io::prelude::*;
use render::Renderer;
use result;
use hyper::client;
use hyper::net::Fresh;
use hyper::method;
use hyper::Url;

pub struct InfluxDbRenderer<'a> {
    time: DateTime<UTC>,
    prefix: Option<String>,
    stream: &'a mut Write,
}

impl<'a> InfluxDbRenderer<'a> {
    pub fn new(time: DateTime<UTC>,
               prefix: Option<String>,
               stream: &'a mut Write)
               -> InfluxDbRenderer<'a> {
        InfluxDbRenderer {
            time: time,
            prefix: prefix,
            stream: stream,
        }
    }
}

impl<'a> Renderer for InfluxDbRenderer<'a> {
    fn render(&mut self, result: result::RequestLogAnalyzerResult) -> () {		
		let client = client::Client::new();
		let res = client.post("http://localhost:8086/write?db=mydb")
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
				count={} \
				time_max={} \
				time_min={} \
				time_avg={} \
				time_median={} \
				time_90percent={}", 
				timing.count,
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
	
	format!("request_log {} {}", timing_values, error_rate_values)
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::io::prelude::*;
    use std::str;
    use chrono::*;
    use analyzer;
    use super::*;

    struct MockTcpStream {
        write_calls: Vec<String>,
    }

    impl Write for MockTcpStream {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.write_calls.push(str::from_utf8(buf).unwrap().to_string());
            Ok(1)
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

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
	fn test_post_body() {
		let result = post_body(get_result_fixture());
		
		assert!(result.contains("time_max=100"));
	}
}
