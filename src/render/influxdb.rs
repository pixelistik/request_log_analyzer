use log::warn;
use crate::render::Renderer;
use crate::result;
use hyper;
use hyper::client;

pub struct InfluxDbRenderer {
    write_url: String,
    tags: Option<String>,
}

impl InfluxDbRenderer {
    pub fn new(write_url: &str, tags: Option<String>) -> InfluxDbRenderer {
        InfluxDbRenderer {
            write_url: String::from(write_url),
            tags,
        }
    }
}

impl Renderer for InfluxDbRenderer {
    fn render(&mut self, result: result::RequestLogAnalyzerResult) -> () {
        let client = client::Client::new();
        let data = self.post_body(result);
        let response = client.post(&self.write_url).body(&data).send().expect(
            "Could not connect to InfluxDB host.",
        );

        if response.status.class() != hyper::status::StatusClass::Success {
            panic!("POSTing data to InfluxDB failed: {:?}", response.status);
        }
    }
}

impl InfluxDbRenderer {
    fn post_body(&self, result: result::RequestLogAnalyzerResult) -> String {
        let mut timing_values = String::from("");
        let mut error_rate_values = String::from("");

        let tags: String = match self.tags {
            Some(ref tags) => format!(",{}", tags),
            None => String::from(""),
        };

        match result.timing {
            Some(timing) => {
                timing_values = format!(
                    ",\
    				time_max={},time_min={},time_avg={},time_median={},\
                                         time_90percent={},time_99percent={}",
                    timing.max,
                    timing.min,
                    timing.avg,
                    timing.median,
                    timing.percentile90,
                    timing.percentile99
                );
            }
            None => warn!("No matching log lines in file."),
        }

        match result.error {
            Some(error) => {
                error_rate_values = format!(
                    ",\
    				client_error_4xx_rate={},server_error_5xx_rate={}",
                    error.client_error_4xx,
                    error.server_error_5xx
                );
            }
            None => warn!("No matching log lines in file."),
        }

        format!(
            "request_log{} count={}{}{}",
            tags,
            result.count,
            timing_values,
            error_rate_values
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::analyzer;
    use super::*;

    fn get_result_fixture() -> result::RequestLogAnalyzerResult {
        result::RequestLogAnalyzerResult {
            count: 3,
            timing: Some(analyzer::TimingResult {
                max: 100,
                min: 1,
                avg: 37,
                median: 10,
                percentile90: 90,
                percentile99: 99,
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
        InfluxDbRenderer::new("http://example.com/write?db=testdb", None);
    }

    #[test]
    fn test_post_body() {
        let renderer = InfluxDbRenderer::new("http://example.com/write?db=testdb", None);
        let result = renderer.post_body(get_result_fixture());

        assert!(result.starts_with("request_log "));
        assert_eq!(result.matches(" ").count(), 1);

        assert!(result.contains("count=3"));
        assert!(result.contains("time_max=100"));
        assert!(result.contains("time_min=1"));
        assert!(result.contains("time_avg=37"));
        assert!(result.contains("time_median=10"));
        assert!(result.contains("time_90percent=90"));
        assert!(result.contains("time_99percent=99"));
        assert!(result.contains("client_error_4xx_rate=0.1"));
        assert!(result.contains("server_error_5xx_rate=0.2"));
    }

    #[test]
    fn test_post_body_with_tag() {
        let tags = String::from("host=testhost");
        let renderer = InfluxDbRenderer::new("http://example.com/write?db=testdb", Some(tags));
        let result = renderer.post_body(get_result_fixture());

        assert!(result.starts_with("request_log,host=testhost "));
        assert_eq!(result.matches(" ").count(), 1);

        assert!(result.contains("count=3"));
    }

    #[test]
    fn test_post_body_empty() {
        let renderer = InfluxDbRenderer::new("http://example.com/write?db=testdb", None);

        let result = renderer.post_body(result::RequestLogAnalyzerResult {
            count: 0,
            timing: None,
            error: None,
        });

        assert!(result.starts_with("request_log "));

        assert!(result.ends_with("count=0"));

        // Don't include empty fields
        assert!(!result.contains("time_max="));
        assert!(!result.contains("time_min="));
        assert!(!result.contains("time_avg="));
        assert!(!result.contains("time_median="));
        assert!(!result.contains("time_90percent="));
        assert!(!result.contains("time_99percent="));
        assert!(!result.contains("client_error_4xx_rate="));
        assert!(!result.contains("server_error_5xx_rate="));
    }
}
