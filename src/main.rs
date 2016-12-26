use std::io;
use std::io::prelude::*;
use std::net::TcpStream;
use std::fs::File;

extern crate chrono;
use chrono::*;

extern crate stats;

extern crate clap;
use clap::{Arg, App, ArgMatches};

mod http_status;
mod log_parser;
use log_parser::log_events::*;
mod request_response_matcher;
use request_response_matcher::RequestResponsePair;
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

fn parse_args<'a>() -> ArgMatches<'a> {
    App::new("Request.log Analyzer")
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
        .get_matches()
}

fn main() {
    let args = parse_args();

    let filename = args.value_of("filename").unwrap_or("-");

    let time_filter = match args.value_of("time_filter_minutes") {
        Some(minutes) => Some(Duration::minutes(minutes.parse().unwrap())),
        None => None
    };

    let mut input: Box<io::Read> = match filename {
        "-" => Box::new(io::stdin()),
        _ => Box::new(File::open(filename).unwrap())
    };


    let reader = io::BufReader::new(input);

    let lines = reader.lines();

    let mut requests: Vec<Request> = Vec::new();
    let mut responses: Vec<Response> = Vec::new();
    let mut pairs: Vec<RequestResponsePair> = Vec::new();

    for line in lines {
        let line_value = &line.unwrap();

        if line_value.contains("->") {
            match Request::new_from_log_line(&line_value) {
                Ok(r) => {
                    // if time_filter.is_none() ||
                    //   (time_filter.is_some() && r.is_between_times(UTC::now().with_timezone(&r.time.timezone()) - time_filter.unwrap(), UTC::now().with_timezone(&r.time.timezone()))) {
                        requests.push(r);
                    // }
                },
                Err(err) => println_stderr!("Skipped a line: {}", err)
            }
        }

        if line_value.contains("<-") {
            match Response::new_from_log_line(&line_value) {
                Ok(r) => responses.push(r),
                Err(err) => println_stderr!("Skipped a line: {}", err)
            }
        }

        // for ref request in &requests {
        if requests.len() == 0 {
            continue;
        }
        for request_index in 0..(requests.len() - 1) {
            let matching_response_index: Option<usize> = responses.iter().position(|response| requests[request_index].id == response.id );

            if matching_response_index.is_some() {
                let request = requests.remove(request_index);
                let response = responses.remove(matching_response_index.unwrap());

                let pair = RequestResponsePair {
                    request: request,
                    response: response
                };
                // println!("request {:?} has response {:?}", request, response);
                // println!("Requests len {:?}", requests.len());
                // println!("========================================", );
                println!("{:?}", pair);
            }
        }
    }

    // let (requests, responses) = log_parser::parse(&mut input);
    //
    // let pairs = request_response_matcher::pair_requests_responses(requests, responses);
    //
    // let conditions = filter::FilterConditions {
    //     include_terms: match args.value_of("include_term") {
    //         Some(value) => Some(vec![value.to_string()]),
    //         None => None
    //     },
    //     exclude_terms: match args.value_of("exclude_term") {
    //         Some(value) => Some(vec![value.to_string()]),
    //         None => None
    //     },
    //     latest_time: time_filter,
    // };
    //
    // let filtered_pairs = filter::filter(pairs, conditions);
    //
    // let result = analyzer::analyze(&filtered_pairs);
    //
    // match result {
    //     Some(result) => {
    //         if args.is_present("graphite-server") {
    //             let stream = TcpStream::connect(
    //                 (
    //                     args.value_of("graphite-server").unwrap(),
    //                     args.value_of("graphite-port").unwrap().parse().unwrap()
    //                 )
    //             ).expect("Could not connect to the Graphite server");
    //
    //             let timezone = filtered_pairs[0].request.time.timezone();
    //
    //             render::render_graphite(result, UTC::now().with_timezone(&timezone), args.value_of("graphite-prefix"), stream);
    //         } else {
    //             render::render_terminal(result);
    //         }
    //     },
    //     None => println_stderr!("No matching log lines in file.")
    // }
}
