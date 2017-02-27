use std::io;
use std::io::prelude::*;
use std::net::TcpStream;
use std::fs::File;
use std::env;

extern crate chrono;
use chrono::*;

#[macro_use]
extern crate clap;

extern crate env_logger;
#[macro_use]
extern crate log;

extern crate stats;

extern crate prometheus;
extern crate hyper;

use hyper::header::ContentType;
use hyper::server;
use hyper::mime::Mime;

mod analyzer;
mod args;
mod filter;
mod log_parser;
use log_parser::log_events::*;
mod render;
use render::Renderer;
mod request_response_matcher;
use request_response_matcher::*;

fn main() {
    env_logger::init().expect("Failed to initialize logging.");

    let args = args::parse_args(env::args()).expect("Failed to parse arguments.");

    fn run(args: &args::RequestLogAnalyzerArgs) -> Option<analyzer::RequestLogAnalyzerResult> {
        let input: Box<io::Read> = match args.filename.as_ref() {
            "-" => Box::new(io::stdin()),
            _ => Box::new(File::open(&args.filename).expect("Failed to open file.")),
        };

        let timings = extract_timings(input, &args.conditions);
        let result = analyzer::analyze(&timings);

        result
    };

    let result = run(&args);
    run(&args);

    let mut stream;
    let mut renderer: Box<render::Renderer>;

    renderer = match args.graphite_server {
        Some(ref graphite_server) => {
            stream = TcpStream::connect((graphite_server.as_ref(), args.graphite_port.unwrap()))
                .expect("Could not connect to the Graphite server");

            Box::new(render::GraphiteRenderer::new(UTC::now(),
                                                   args.graphite_prefix.clone(),
                                                   &mut stream))
        }
        None => Box::new(render::TerminalRenderer::new()),
    };

    match result {
        Some(result) => {
            renderer.render(result);
        }
        None => warn!("No matching log lines in file."),
    }

    let addr = "127.0.0.1:9898";
    println!("listening addr {:?}", addr);
    hyper::server::Server::http(addr)
        .unwrap()
        .handle(move |_: hyper::server::Request, mut res: hyper::server::Response| {

            let result = run(&args);

            let mut renderer = render::prometheus::PrometheusRenderer::new();
            renderer.render(result.unwrap());
            // res.headers_mut()
            //     .set(ContentType(encoder.format_type().parse::<Mime>().unwrap()));
            res.send(&renderer.buffer).unwrap();
        })
        .unwrap();
}

fn extract_timings(input: Box<io::Read>, conditions: &filter::FilterConditions) -> Vec<i64> {
    let reader = io::BufReader::new(input);

    let mut requests: Vec<Request> = Vec::new();
    let mut responses: Vec<Response> = Vec::new();
    let mut timings: Vec<i64> = Vec::new();

    for line in reader.lines() {
        let parsed_line = log_parser::parse_line(&line.expect("Failed to read line."));

        match parsed_line {
            Ok(event) => {
                match event {
                    LogEvent::Request(request) => requests.push(request),
                    LogEvent::Response(response) => responses.push(response),
                }

                let pairs = extract_matching_request_response_pairs(&mut requests, &mut responses);

                let mut new_timings: Vec<i64> = pairs.iter()
                    .filter(|pair| filter::matches_filter(&pair, conditions))
                    .map(|pair| pair.response.response_time.num_milliseconds())
                    .collect();

                timings.append(&mut new_timings);
            }
            Err(err) => warn!("{}", err),
        }
    }
    timings
}

#[test]
fn test_extract_timings() {
    let conditions = filter::FilterConditions {
        include_terms: None,
        exclude_terms: None,
        latest_time: None,
    };

    let timings = extract_timings(Box::new(File::open("src/test/simple-1.log").unwrap()),
                                  &conditions);

    assert_eq!(timings, vec![7, 10]);
}

#[test]
fn test_extract_timings_ignore_broken_lines() {
    let conditions = filter::FilterConditions {
        include_terms: None,
        exclude_terms: None,
        latest_time: None,
    };

    let timings = extract_timings(Box::new(File::open("src/test/broken.log").unwrap()),
                                  &conditions);

    assert_eq!(timings, vec![7]);
}
