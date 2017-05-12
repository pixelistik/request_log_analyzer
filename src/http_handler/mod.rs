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
    use std::io::{self, Read, Write, Cursor};
    use std::str;
    use std::net::Shutdown;
    use std::time::Duration;
    use std::cell::Cell;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use hyper::net::NetworkStream;
    use hyper::server::Handler;
    use hyper;

    use filter;
    use analyzer;
    use super::*;

    // MockStream is copied from Hyper tests
    // https://github.com/hyperium/hyper/blob/0.10.x/src/mock.rs
    // MIT licensed, Copyright (c) 2014 Sean McArthur
    #[derive(Clone, Debug)]
    pub struct MockStream {
        pub read: Cursor<Vec<u8>>,
        next_reads: Vec<Vec<u8>>,
        pub write: Vec<u8>,
        pub is_closed: bool,
        pub error_on_write: bool,
        pub error_on_read: bool,
        pub read_timeout: Cell<Option<Duration>>,
        pub write_timeout: Cell<Option<Duration>>,
    }

    impl PartialEq for MockStream {
        fn eq(&self, other: &MockStream) -> bool {
            self.read.get_ref() == other.read.get_ref() && self.write == other.write
        }
    }

    impl MockStream {
        pub fn new() -> MockStream {
            MockStream::with_input(b"")
        }

        pub fn with_input(input: &[u8]) -> MockStream {
            MockStream::with_responses(vec![input])
        }

        pub fn with_responses(mut responses: Vec<&[u8]>) -> MockStream {
            MockStream {
                read: Cursor::new(responses.remove(0).to_vec()),
                next_reads: responses.into_iter().map(|arr| arr.to_vec()).collect(),
                write: vec![],
                is_closed: false,
                error_on_write: false,
                error_on_read: false,
                read_timeout: Cell::new(None),
                write_timeout: Cell::new(None),
            }
        }
    }

    impl Read for MockStream {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            if self.error_on_read {
                Err(io::Error::new(io::ErrorKind::Other, "mock error"))
            } else {
                match self.read.read(buf) {
                    Ok(n) => {
                        if self.read.position() as usize == self.read.get_ref().len() {
                            if self.next_reads.len() > 0 {
                                self.read = Cursor::new(self.next_reads.remove(0));
                            }
                        }
                        Ok(n)
                    }
                    r => r,
                }
            }
        }
    }

    impl Write for MockStream {
        fn write(&mut self, msg: &[u8]) -> io::Result<usize> {
            if self.error_on_write {
                Err(io::Error::new(io::ErrorKind::Other, "mock error"))
            } else {
                Write::write(&mut self.write, msg)
            }
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    impl NetworkStream for MockStream {
        fn peer_addr(&mut self) -> io::Result<SocketAddr> {
            Ok("127.0.0.1:1337".parse().unwrap())
        }

        fn set_read_timeout(&self, dur: Option<Duration>) -> io::Result<()> {
            self.read_timeout.set(dur);
            Ok(())
        }

        fn set_write_timeout(&self, dur: Option<Duration>) -> io::Result<()> {
            self.write_timeout.set(dur);
            Ok(())
        }

        fn close(&mut self, _how: Shutdown) -> io::Result<()> {
            self.is_closed = true;
            Ok(())
        }
    }

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

        fn run_fn(_: &args::RequestLogAnalyzerArgs) -> Option<analyzer::RequestLogAnalyzerResult> {
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

        // Create a minimal HTTP request
        let mut request_mock_network_stream = MockStream::with_input(b"GET / HTTP/1.0\r\n\r\n");

        let mut reader = hyper::buffer::BufReader::new(&mut request_mock_network_stream as
                                                       &mut hyper::net::NetworkStream);

        let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);

        let request = hyper::server::Request::new(&mut reader, socket).unwrap();

        let mut headers = hyper::header::Headers::new();
        let mut response_mock_network_stream = MockStream::new();

        {
            let response = hyper::server::Response::new(&mut response_mock_network_stream,
                                                        &mut headers);

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
