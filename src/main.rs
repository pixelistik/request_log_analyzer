use std::io;
use std::io::prelude::*;
use std::net::TcpStream;
use std::fs::File;
use std::env;

extern crate chrono;
use chrono::*;

extern crate aggregated_stats;

#[macro_use]
extern crate clap;

extern crate env_logger;
#[macro_use]
extern crate log;

extern crate prometheus;
extern crate hyper;

mod analyzer;
mod args;
mod filter;
mod log_parser;
use log_parser::log_events::*;
mod render;
mod request_response_matcher;
mod http_handler;
mod result;

fn main() {
    env_logger::init().expect("Failed to initialize logging.");

    let args = args::parse_args(env::args()).expect("Failed to parse arguments.");

    let result = run(&args);

    let mut stream;
    let mut renderer: Box<render::Renderer>;

    renderer = match args.graphite_server {
        Some(ref graphite_server) => {
            stream = TcpStream::connect((graphite_server.as_ref(), args.graphite_port.unwrap()))
                .expect("Could not connect to the Graphite server");

            Box::new(render::graphite::GraphiteRenderer::new(UTC::now(),
                                                             args.graphite_prefix.clone(),
                                                             &mut stream))
        }
        None => Box::new(render::TerminalRenderer::new()),
    };

    renderer.render(result);

    if args.prometheus_listen.is_some() {
        let binding_address = args.prometheus_listen.clone().unwrap();
        http_handler::listen_http(args, &binding_address);
    }

}

fn run(args: &args::RequestLogAnalyzerArgs) -> result::RequestLogAnalyzerResult {
    let input: Box<io::Read> = match args.filename.as_ref() {
        "-" => Box::new(io::stdin()),
        _ => Box::new(File::open(&args.filename).expect("Failed to open file.")),
    };

    let reader = io::BufReader::new(input);

    let mut events_iterator = reader.lines()
        .map(parse_event)
        .filter(|event| event.is_ok())
        .map(|event| event.unwrap());

    let pairs_iterator =
        request_response_matcher::RequestResponsePairIterator::new(&mut events_iterator)
            .filter(|pair| filter::matches_filter(pair, &args.conditions));

    analyzer::analyze_iterator(pairs_iterator)
}

fn parse_event(line: Result<String, std::io::Error>) -> Result<LogEvent, &'static str> {
    match line {
        Ok(line) => log_parser::parse_line(&line),
        Err(_) => Err("Failed to read line."),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run() {
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

        let result = run(&args);
        assert_eq!(result.count, 2);

        let timing = result.timing.unwrap();
        assert_eq!(timing.min, 7);
        assert_eq!(timing.max, 10);

        assert!(result.error.is_some());
    }

    #[test]
    fn test_run_ignore_broken_lines() {
        let args = args::RequestLogAnalyzerArgs {
            filename: String::from("src/test/broken.log"),
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

        let result = run(&args);
        assert_eq!(result.count, 1);
    }

    #[test]
    fn test_parse_event() {
        let lines = vec![Ok(String::from("08/Apr/2016:09:57:47 +0200 [001] -> GET \
                                          /content/some/page.html HTTP/1.1")),
                         Ok(String::from("08/Apr/2016:09:57:47 +0200 [001] <- 200 text/html 7ms"))];

        let result: Vec<Result<LogEvent, &str>> = lines.into_iter().map(parse_event).collect();
        assert_eq!(result.len(), 2);
    }
}
