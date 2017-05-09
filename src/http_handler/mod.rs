use args;
use analyzer;
use hyper;
use prometheus::Encoder;
use render;
use render::Renderer;
use run;

struct HttpHandler {
    args: args::RequestLogAnalyzerArgs,
    run: fn(&args::RequestLogAnalyzerArgs) -> Option<analyzer::RequestLogAnalyzerResult>,
}

impl hyper::server::Handler for HttpHandler {
    fn handle(&self, _: hyper::server::Request, mut res: hyper::server::Response) {
        let result = (self.run)(&self.args);

        let mut renderer = render::prometheus::PrometheusRenderer::new();
        renderer.render(result);
        res.headers_mut()
            .set(hyper::header::ContentType(renderer.encoder
                .format_type()
                .parse::<hyper::mime::Mime>()
                .unwrap()));
        res.send(&renderer.buffer).unwrap();
    }
}

pub fn listen_http(args: args::RequestLogAnalyzerArgs, binding_address: &str) {
    let handler = HttpHandler {
        args: args,
        run: run,
    };

    info!("listening addr {:?}", binding_address);
    hyper::server::Server::http(binding_address).unwrap().handle(handler).unwrap();
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::io::prelude::*;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4};
    use std::time::Duration;
    use hyper::server::Handler;
    use hyper;
    use filter;
    use analyzer;
    use super::*;

    #[test]
    fn test_handle() {
        let args = args::RequestLogAnalyzerArgs {
            filename: String::from("src/test/simple-1.log"),
            conditions: filter::FilterConditions {
                include_terms: None,
                exclude_terms: None,
                latest_time: None,
            },
            graphite_server: None,
            graphite_port: Some(2003),
            graphite_prefix: None,
            prometheus_listen: None,
        };

        fn run_fn(args: &args::RequestLogAnalyzerArgs)
                  -> Option<analyzer::RequestLogAnalyzerResult> {
            Some(analyzer::RequestLogAnalyzerResult {
                count: 3,
                max: 100,
                min: 1,
                avg: 37,
                median: 10,
                percentile90: 100,
            })
        };

        let handler = HttpHandler {
            args: args,
            run: run_fn,
        };

        struct MockNetworkStream {}

        impl Read for MockNetworkStream {
            fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
                Ok(1)
            }
        }

        impl Write for MockNetworkStream {
            fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
                Ok(1)
            }

            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }

        impl hyper::net::NetworkStream for MockNetworkStream {
            fn peer_addr(&mut self) -> Result<SocketAddr, io::Error> {
                Ok(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080)))
            }

            fn set_read_timeout(&self, dur: Option<Duration>) -> Result<(), io::Error> {
                Ok(())
            }

            fn set_write_timeout(&self, dur: Option<Duration>) -> Result<(), io::Error> {
                Ok(())
            }
        }

        let mut request_mock_network_stream = MockNetworkStream {};

        let mut reader = hyper::buffer::BufReader::new(&mut request_mock_network_stream as
                                                       &mut hyper::net::NetworkStream);

        let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let request = hyper::server::Request::new(&mut reader, socket).unwrap();

        let mut headers = hyper::header::Headers::new();
        let mut response_mock_network_stream = MockNetworkStream {};
        let response = hyper::server::Response::new(&mut response_mock_network_stream,
                                                    &mut headers);
        handler.handle(request, response);
    }
}
