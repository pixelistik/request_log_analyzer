use std::io;
use std::io::prelude::*;
use std::net::TcpStream;
use std::fs::File;
use std::env;

extern crate chrono;
use chrono::*;

extern crate stats;

extern crate clap;
use clap::{Arg, App, ArgMatches};

mod http_status;
mod log_parser;
use log_parser::parse_line;
use log_parser::log_events::*;
mod request_response_matcher;
use request_response_matcher::*;
mod filter;
mod analyzer;
mod render;

// http://stackoverflow.com/a/27590832/376138
macro_rules! println_stderr(
    ($($arg:tt)*) => { {
        let r = writeln!(&mut ::std::io::stderr(), $($arg)*);
        r.expect("failed printing to stderr");
    } }
);

#[derive(Debug)]
#[derive(PartialEq)]
struct RequestLogAnalyzerArgs {
    filename: String,
    conditions: filter::FilterConditions,

}

fn parse_args<'a, T>(args: T) -> Result<RequestLogAnalyzerArgs, &'a str> where T: IntoIterator<Item=String> {
    let app = App::new("Request.log Analyzer")
        .arg(Arg::with_name("filename")
            .index(1)
            .value_name("FILE")
            .required(false)
            .help("Log file to analyze, defaults to stdin")
            .takes_value(true))
        .arg(Arg::with_name("time_filter_minutes")
            .value_name("MINUTES")
            .short("t")
            .help("Limit to the last n minutes")
            .takes_value(true))
        .arg(Arg::with_name("include_term")
            .value_name("TERM")
            .long("include")
            .help("Only includes lines that contain this term")
            .takes_value(true))
        .arg(Arg::with_name("exclude_term")
            .value_name("TERM")
            .long("exclude")
            .help("Excludes lines that contain this term")
            .takes_value(true))
        .arg(Arg::with_name("graphite-server")
            .value_name("GRAPHITE_SERVER")
            .long("graphite-server")
            .help("Send values to this Graphite server instead of stdout")
            .takes_value(true))
        .arg(Arg::with_name("graphite-port")
            .value_name("GRAPHITE_PORT")
            .long("graphite-port")
            .takes_value(true)
            .default_value("2003"))
        .arg(Arg::with_name("graphite-prefix")
            .value_name("GRAPHITE_PREFIX")
            .long("graphite-prefix")
            .help("Prefix for Graphite key, e.g. 'servers.prod.publisher1'")
            .takes_value(true))
        .get_matches_from(args);

        let filename = app.value_of("filename").unwrap_or("-").to_string();

        let conditions = filter::FilterConditions {
            include_terms: match app.value_of("include_term") {
                Some(value) => Some(vec![value.to_string()]),
                None => None,
            },
            exclude_terms: match app.value_of("exclude_term") {
                Some(value) => Some(vec![value.to_string()]),
                None => None,
            },
            latest_time: match app.value_of("time_filter_minutes") {
                Some(minutes) => Some(Duration::minutes(minutes.parse().unwrap())),
                None => None,
            },
        };

        Ok(RequestLogAnalyzerArgs {
            filename: filename,
            conditions: conditions,
        })
}

fn main() {
    let args = parse_args(env::args()).unwrap();

    let input: Box<io::Read> = match args.filename.as_ref() {
        "-" => Box::new(io::stdin()),
        _ => Box::new(File::open(&args.filename).unwrap()),
    };

    let reader = io::BufReader::new(input);
    let lines = reader.lines();

    let mut requests: Vec<Request> = Vec::new();
    let mut responses: Vec<Response> = Vec::new();
    let mut times: Vec<i64> = Vec::new();

    // We need to store 1 Request in order to determine the timezone later
    let mut first_request: Option<Request> = None;

    for line in lines {
        let line_value = &line.unwrap();

        let event = parse_line(line_value).unwrap();

        match event {
            LogEvent::Request(request) => {
                if first_request.is_none() {
                    first_request = Some(request.clone());
                }
                requests.push(request)
            }
            LogEvent::Response(response) => responses.push(response),
        }

        let pairs = extract_matching_request_response_pairs(&mut requests, &mut responses);

        let mut new_times: Vec<i64> = pairs.iter()
            .filter(|pair| filter::matches_filter(&pair, &args.conditions))
            .map(|pair| pair.response.response_time.num_milliseconds())
            .collect();

        times.append(&mut new_times);
    }

    let result = analyzer::analyze(&times);

    match result {
        Some(result) => {
            if args.is_present("graphite-server") {
                let stream =
                    TcpStream::connect((args.value_of("graphite-server").unwrap(),
                                        args.value_of("graphite-port").unwrap().parse().unwrap()))
                        .expect("Could not connect to the Graphite server");

                let timezone = first_request.unwrap().time.timezone();

                render::render_graphite(result,
                                        UTC::now().with_timezone(&timezone),
                                        args.value_of("graphite-prefix"),
                                        stream);
            } else {
                render::render_terminal(result);
            }
        }
        None => println_stderr!("No matching log lines in file."),
    }
}

#[test]
fn test_parse_args_default() {
    let raw_args = vec!["request_log_analyzer".to_string()];

    let expected = RequestLogAnalyzerArgs {
        filename: String::from("-"),
        conditions: filter::FilterConditions {
            include_terms: None,
            exclude_terms: None,
            latest_time: None,
        },
    };

    let result = parse_args(raw_args).unwrap();

    assert_eq!(result, expected);
}

#[test]
fn test_parse_args_all() {
    let raw_args = vec![String::from("request_log_analyzer"),
                        String::from("--include"), String::from("one"),
                        String::from("--exclude"), String::from("this other"),
                        String::from("-t"), String::from("10"),
                        String::from("my-logfile.log"),];

    let expected = RequestLogAnalyzerArgs {
        filename: String::from("my-logfile.log"),
        conditions: filter::FilterConditions {
            include_terms: Some(vec![String::from("one")]),
            exclude_terms: Some(vec![String::from("this other")]),
            latest_time: Some(Duration::minutes(10)),
        },
    };

    let result = parse_args(raw_args).unwrap();

    assert_eq!(result, expected);
}
