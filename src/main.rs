use std::io::{self, BufReader, Write};
use std::io::BufRead;
use std::net::TcpStream;
use std::fs::File;

extern crate chrono;
use chrono::*;

extern crate stats;
use stats::median;

extern crate clap;
use clap::{Arg, App, ArgMatches};

mod percentile;
use percentile::percentile;

mod http_status;

mod request_response;
use request_response::*;

fn open_logfile(path: &str) -> BufReader<File> {
    let file = File::open(path);

    match file {
        Ok(f) => BufReader::new(f),
        Err(err) => panic!("Could not open file {}: {}", path, err)
    }
}

pub fn parse_logfile(path: &str, time_filter: Option<Duration>, exclude_term: Option<&str>) -> Result<(Vec<Request>,Vec<Response>), &'static str> {
    let f = open_logfile(path);

    let mut requests: Vec<Request> = Vec::new();
    let mut responses: Vec<Response> = Vec::new();

    for line in f.lines() {
        let line_value = &line.unwrap();

        if exclude_term.is_some() && line_value.contains(exclude_term.unwrap()) {
            continue;
        }

        if line_value.contains("->") {
            let r = try!(Request::new_from_log_line(&line_value, None));

            if time_filter.is_none() ||
              (time_filter.is_some() && r.is_between_times(UTC::now().with_timezone(&r.time.timezone()) - time_filter.unwrap(), UTC::now().with_timezone(&r.time.timezone()))) {
                requests.push(r);
            }
        }

        if line_value.contains("<-") {
            let r = try!(Response::new_from_log_line(&line_value, None));
            responses.push(r);
        }

    }

    responses.sort_by_key(|r| r.id);

    Ok((requests, responses))
}

#[derive(Eq, PartialEq)]
#[derive(Debug)]
pub struct RequestLogAnalyzerResult {
    count: usize,
    max: usize,
    min: usize,
    avg: usize,
    median: usize,
    percentile90: usize,
}

pub fn analyze(request_response_pairs: &Vec<RequestResponsePair>) -> Option<RequestLogAnalyzerResult> {
    if request_response_pairs.len() == 0 {
        return None;
    }

    let times: Vec<i64> = request_response_pairs.iter()
        .map(|rr: &RequestResponsePair| -> i64 {rr.response.response_time.num_milliseconds() })
        .collect();

    let sum: usize = times.iter().sum::<i64>() as usize;
    let avg: usize = sum / times.len();

    let max: usize = *times.iter().max().unwrap() as usize;
    let min: usize = *times.iter().min().unwrap() as usize;

    let percentile90: usize = percentile(&times, 0.9) as usize;

    let median = median(times.into_iter()).unwrap() as usize;

    Some(RequestLogAnalyzerResult {
        count: request_response_pairs.len().into(),
        max: max,
        min: min,
        avg: avg,
        median: median,
        percentile90: percentile90,
    })
}

fn render_terminal(result: RequestLogAnalyzerResult) {
    println!("count:\t{}", result.count);
    println!("time.avg:\t{}", result.avg);
    println!("time.min:\t{}", result.min);
    println!("time.median:\t{}", result.median);
    println!("time.90percent:\t{}", result.percentile90);
    println!("time.max:\t{}", result.max);
}

pub fn render_graphite<T: Write>(result: RequestLogAnalyzerResult, time: DateTime<FixedOffset>, prefix: Option<&str>, mut stream: T) {
    let prefix_text: &str;
    let prefix_separator: &str;

    match prefix {
        Some(p) => {
            prefix_text = p;
            prefix_separator = ".";
        }
        None => {
            prefix_text = "";
            prefix_separator = "";
        }
    };

    let mut write = |text: String| {
        stream.write(
            format!("{}{}{} {}\n", prefix_text, prefix_separator, text, time.timestamp() )
            .as_bytes()
        );
    };

    write(format!("requests.count {}", result.count));
    write(format!("requests.time.max {}", result.max));
    write(format!("requests.time.min {}", result.min));
    write(format!("requests.time.avg {}", result.avg));
    write(format!("requests.time.median {}", result.median));
    write(format!("requests.time.90percent {}", result.percentile90));
}

fn parse_args<'a>() -> ArgMatches<'a> {
    App::new("Request.log Analyzer")
        .arg(Arg::with_name("filename")
            .index(1)
            .value_name("FILE")
            .required(true)
            .help("Log file to analyze")
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

    let filename = args.value_of("filename").unwrap();

    let time_filter = match args.value_of("time_filter_minutes") {
        Some(minutes) => Some(Duration::minutes(minutes.parse().unwrap())),
        None => None
    };

    let lines = parse_logfile(filename, time_filter, args.value_of("exclude_term"));
    let (requests, responses) = lines.unwrap();

    let time_zone = &requests[0].time.timezone();

    let pairs: Vec<RequestResponsePair> = pair_requests_responses(requests, responses)
        .into_iter()
        .filter(|rr| rr.matches_include_filter())
        .collect();

    if args.is_present("graphite-server") {
        let stream = TcpStream::connect(
            (
                args.value_of("graphite-server").unwrap(),
                args.value_of("graphite-port").unwrap().parse().unwrap()
            )
        ).unwrap();

        match analyze(&pairs) {
            Some(result) => render_graphite(result, UTC::now().with_timezone(time_zone), args.value_of("graphite-prefix"), stream),
            None => println!("No matching log lines in file.")
        }
    } else {
        match analyze(&pairs) {
            Some(result) => render_terminal(result),
            None => println!("No matching log lines in file.")
        }
    }
}

#[cfg(test)]
mod tests {
	use super::*;
    use request_response::*;
    extern crate chrono;
    use chrono::*;
    use std::str;
    use std::io::prelude::Write;
    use std::io::{self, BufReader};

    #[test]
    fn test_parse_logfile() {
        let lines = parse_logfile("src/test/simple-1.log", None, None);
        let (requests, responses) = lines.unwrap();

        assert_eq!(requests.len(), 2);
        assert_eq!(responses.len(), 2);
    }

    #[test]
    fn test_open_logfile_time_filter() {
        let time_filter: Duration = Duration::minutes(1);
        let lines = parse_logfile("src/test/simple-1.log", Some(time_filter), None);
        let (requests, responses) = lines.unwrap();

        assert_eq!(requests.len(), 0);

        let time_filter: Duration = Duration::minutes(52560000); // 100 years
        let lines = parse_logfile("src/test/simple-1.log", Some(time_filter), None);
        let (requests, responses) = lines.unwrap();

        assert_eq!(requests.len(), 2);
    }

    #[test]
    fn test_parse_logfile_exlude_term_in_request_line() {
        let lines = parse_logfile("src/test/simple-1.log", None, Some("other.html"));
        let (requests, responses) = lines.unwrap();

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].id, 1);
    }

    #[test]
    fn test_parse_logfile_exlude_term_in_response_line() {
        let lines = parse_logfile("src/test/simple-1.log", None, Some("text/html"));
        let (requests, responses) = lines.unwrap();

        assert_eq!(responses.len(), 0);
    }

    #[test]
    fn test_parse_logfile_exlude_term_given_but_not_found() {
        let lines = parse_logfile("src/test/simple-1.log", None, Some("term that does not exist"));
        let (requests, responses) = lines.unwrap();

        assert_eq!(requests.len(), 2);
        assert_eq!(responses.len(), 2);
    }

    #[test]
    fn test_pair_requests_resonses() {
        let lines = parse_logfile("src/test/simple-1.log", None, None);
        let (requests, responses) = lines.unwrap();

        let result = pair_requests_responses(requests, responses);

        assert_eq!(result.len(), 2);

        assert_eq!(result[0].request.id, result[0].response.id);
        assert_eq!(result[1].request.id, result[1].response.id);
    }

    #[test]
    fn test_request_log_analyzer_result() {
        let lines = parse_logfile("src/test/response-time-calculations.log", None, None);
        let (requests, responses) = lines.unwrap();

        let request_response_pairs = pair_requests_responses(requests, responses);

        let result = analyze(&request_response_pairs);

        let expected = Some(RequestLogAnalyzerResult {
            count: 3,
            max: 100,
            min: 1,
            avg: 37,
            median: 10,
            percentile90: 100,
        });

        assert_eq!(result, expected);
    }

    #[test]
    fn test_request_log_analyze_none_matching() {
        let lines = parse_logfile("src/test/simple-1.log", Some(Duration::minutes(0)), None);
        let (requests, responses) = lines.unwrap();

        let request_response_pairs = pair_requests_responses(requests, responses);

        let result = analyze(&request_response_pairs);

        let expected = None;

        assert_eq!(result, expected);
    }

    #[test]
    fn test_90_percentile_calculation() {
        let lines = parse_logfile("src/test/percentile.log", None, None);
        let (requests, responses) = lines.unwrap();

        let request_response_pairs = pair_requests_responses(requests, responses);

        let result: RequestLogAnalyzerResult = analyze(&request_response_pairs).unwrap();

        assert_eq!(result.percentile90, 9);
    }

    struct MockTcpStream {
        write_calls: Vec<String>,
    }

    impl Write for MockTcpStream {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.write_calls.push(str::from_utf8(buf).unwrap().to_string());
            Ok(1)
        }

        fn flush(&mut self) -> io::Result<()> { Ok(()) }
    }

    #[test]
    fn test_render_graphite() {
        let mut mock_tcp_stream = MockTcpStream {
            write_calls: vec![]
        };

        render_graphite(RequestLogAnalyzerResult {
                count: 3,
                max: 100,
                min: 1,
                avg: 37,
                median: 10,
                percentile90: 100,
            },
            DateTime::parse_from_str("22/Sep/2016:22:41:59 +0200", "%d/%b/%Y:%H:%M:%S %z").unwrap(),
            None,
            &mut mock_tcp_stream
        );

        assert_eq!(&mock_tcp_stream.write_calls[0], "requests.count 3 1474576919\n");
        assert_eq!(&mock_tcp_stream.write_calls[1], "requests.time.max 100 1474576919\n");
        assert_eq!(&mock_tcp_stream.write_calls[2], "requests.time.min 1 1474576919\n");
        assert_eq!(&mock_tcp_stream.write_calls[3], "requests.time.avg 37 1474576919\n");
        assert_eq!(&mock_tcp_stream.write_calls[4], "requests.time.median 10 1474576919\n");
        assert_eq!(&mock_tcp_stream.write_calls[5], "requests.time.90percent 100 1474576919\n");
    }

    #[test]
    fn test_render_graphite_prefix() {
        let mut mock_tcp_stream = MockTcpStream {
            write_calls: vec![]
        };

        render_graphite(RequestLogAnalyzerResult {
                count: 3,
                max: 100,
                min: 1,
                avg: 37,
                median: 10,
                percentile90: 100,
            },
            DateTime::parse_from_str("22/Sep/2016:22:41:59 +0200", "%d/%b/%Y:%H:%M:%S %z").unwrap(),
            Some("my.prefix"),
            &mut mock_tcp_stream
        );

        assert_eq!(&mock_tcp_stream.write_calls[0], "my.prefix.requests.count 3 1474576919\n");
        assert_eq!(&mock_tcp_stream.write_calls[1], "my.prefix.requests.time.max 100 1474576919\n");
        assert_eq!(&mock_tcp_stream.write_calls[2], "my.prefix.requests.time.min 1 1474576919\n");
        assert_eq!(&mock_tcp_stream.write_calls[3], "my.prefix.requests.time.avg 37 1474576919\n");
        assert_eq!(&mock_tcp_stream.write_calls[4], "my.prefix.requests.time.median 10 1474576919\n");
        assert_eq!(&mock_tcp_stream.write_calls[5], "my.prefix.requests.time.90percent 100 1474576919\n");
    }
}
