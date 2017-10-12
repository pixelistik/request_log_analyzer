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
		
		let body = format!("request_log {} {}", timing_values, error_rate_values);
		
		let client = client::Client::new();
		let res = client.post("http://localhost:8086/write?db=mydb")
					.body(&body)
					.send()
					.unwrap();
    }
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

    fn get_time_fixture() -> DateTime<UTC> {
        let time: DateTime<UTC> = DateTime::parse_from_str("22/Sep/2016:22:41:59 +0200",
                                                           "%d/%b/%Y:%H:%M:%S %z")
            .unwrap()
            .with_timezone(&UTC);

        time
    }

    #[test]
    fn test_render_graphite() {
        let mut mock_tcp_stream = MockTcpStream { write_calls: vec![] };

        {
            let mut renderer =
                InfluxDbRenderer::new(get_time_fixture(), None, &mut mock_tcp_stream);
            renderer.render(get_result_fixture());
        }

        assert_eq!(&mock_tcp_stream.write_calls[0],
                   "requests.count 3 1474576919\n");
        assert_eq!(&mock_tcp_stream.write_calls[1],
                   "requests.time.max 100 1474576919\n");
        assert_eq!(&mock_tcp_stream.write_calls[2],
                   "requests.time.min 1 1474576919\n");
        assert_eq!(&mock_tcp_stream.write_calls[3],
                   "requests.time.avg 37 1474576919\n");
        assert_eq!(&mock_tcp_stream.write_calls[4],
                   "requests.time.median 10 1474576919\n");
        assert_eq!(&mock_tcp_stream.write_calls[5],
                   "requests.time.90percent 100 1474576919\n");
        assert_eq!(&mock_tcp_stream.write_calls[6],
                   "requests.error.client_error_4xx_rate 0.1 1474576919\n");
        assert_eq!(&mock_tcp_stream.write_calls[7],
                   "requests.error.server_error_5xx_rate 0.2 1474576919\n");
    }

    #[test]
    fn test_render_graphite_with_prefix() {
        let mut mock_tcp_stream = MockTcpStream { write_calls: vec![] };

        {
            let mut renderer = InfluxDbRenderer::new(get_time_fixture(),
                                                     Some(String::from("my_prefix")),
                                                     &mut mock_tcp_stream);
            renderer.render(get_result_fixture());
        }

        assert_eq!(&mock_tcp_stream.write_calls[0],
                   "my_prefix.requests.count 3 1474576919\n");
        assert_eq!(&mock_tcp_stream.write_calls[1],
                   "my_prefix.requests.time.max 100 1474576919\n");
        assert_eq!(&mock_tcp_stream.write_calls[2],
                   "my_prefix.requests.time.min 1 1474576919\n");
        assert_eq!(&mock_tcp_stream.write_calls[3],
                   "my_prefix.requests.time.avg 37 1474576919\n");
        assert_eq!(&mock_tcp_stream.write_calls[4],
                   "my_prefix.requests.time.median 10 1474576919\n");
        assert_eq!(&mock_tcp_stream.write_calls[5],
                   "my_prefix.requests.time.90percent 100 1474576919\n");
        assert_eq!(&mock_tcp_stream.write_calls[6],
                   "my_prefix.requests.error.client_error_4xx_rate 0.1 1474576919\n");
        assert_eq!(&mock_tcp_stream.write_calls[7],
                   "my_prefix.requests.error.server_error_5xx_rate 0.2 1474576919\n");
    }
}
