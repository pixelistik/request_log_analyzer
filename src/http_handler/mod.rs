use args;
use result;
use hyper;
use prometheus::Encoder;
use render;
use render::Renderer;
use run;

struct HttpHandler {
    args: args::RequestLogAnalyzerArgs,
    run: fn(&args::RequestLogAnalyzerArgs) -> result::RequestLogAnalyzerResult,
}

impl hyper::server::Handler for HttpHandler {
    fn handle(&self, _: hyper::server::Request, mut res: hyper::server::Response) {
        let result = (self.run)(&self.args);

        let mut renderer = render::prometheus::PrometheusRenderer::new();
        renderer.render(result);
        res.headers_mut().set(hyper::header::ContentType(
            renderer
                .encoder
                .format_type()
                .parse::<hyper::mime::Mime>()
                .unwrap(),
        ));
        res.send(&renderer.buffer).unwrap();
    }
}

pub fn listen_http(args: args::RequestLogAnalyzerArgs, binding_address: &str) {
    let handler = HttpHandler { args, run };

    info!("listening addr {:?}", binding_address);
    hyper::server::Server::http(binding_address)
        .unwrap()
        .handle(handler)
        .unwrap();
}

#[cfg(test)]
mod tests {
    use std::str;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use hyper::server::Handler;
    use hyper;

    use filter;
    use analyzer;
    use super::*;

    mod mock;

    #[test]
    fn test_handle() {
        let args = args::RequestLogAnalyzerArgs {
            filenames: vec![String::from("src/test/simple-1.log")],
            conditions: filter::FilterConditions {
                include_terms: None,
                exclude_terms: None,
                latest_time: None,
            },
            graphite_server: None,
            graphite_port: Some(2003),
            graphite_prefix: None,
            prometheus_listen: None,
            influxdb_write_url: None,
            influxdb_tags: None,
            quiet: false,
        };

        fn run_fn(_: &args::RequestLogAnalyzerArgs) -> result::RequestLogAnalyzerResult {
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
                error: None,
            }
        };

        let handler = HttpHandler {
            args: args,
            run: run_fn,
        };

        // Create a minimal HTTP request
        let mut request_mock_network_stream =
            mock::MockStream::with_input(b"GET / HTTP/1.0\r\n\r\n");

        let mut reader = hyper::buffer::BufReader::new(
            &mut request_mock_network_stream as &mut hyper::net::NetworkStream,
        );

        let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);

        let request = hyper::server::Request::new(&mut reader, socket).unwrap();

        let mut headers = hyper::header::Headers::new();
        let mut response_mock_network_stream = mock::MockStream::new();

        {
            let response =
                hyper::server::Response::new(&mut response_mock_network_stream, &mut headers);

            handler.handle(request, response);
        }

        let result = str::from_utf8(&response_mock_network_stream.write).unwrap();

        assert!(result.contains("request_count 3"));
        assert!(result.contains("request_time_max 100"));
        assert!(result.contains("request_time_min 1"));
        assert!(result.contains("request_time_avg 37"));
        assert!(result.contains("request_time_median 10"));
        assert!(result.contains("request_time_percentile90 100"));

    }
}
