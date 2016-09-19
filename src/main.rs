use std::io::{self, BufReader};
use std::io::BufRead;
use std::fs::File;

extern crate time;
use time::Duration;

extern crate stats;
use stats::median;

extern crate clap;
use clap::{Arg, App};

mod percentile;
use percentile::percentile;

mod http_status;

mod request_response;
use request_response::*;

pub fn open_logfile(path: &str, time_filter: Option<Duration>) -> Result<(Vec<Request>,Vec<Response>), io::Error> {
    let f = try!(File::open(path));

    let f = BufReader::new(f);

    let mut requests: Vec<Request> = Vec::new();
    let mut responses: Vec<Response> = Vec::new();

    for line in f.lines() {
        let line_value = &line.unwrap();

        if line_value.contains("->") {
            let r = try!(Request::new_from_log_line(&line_value));

            if time_filter.is_none() ||
              (time_filter.is_some() && r.is_between_times(time::now() - time_filter.unwrap(), time::now())) {
                requests.push(r);
            }
        }

        if line_value.contains("<-") {
            let r = try!(Response::new_from_log_line(&line_value));
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

pub fn analyze(request_response_pairs: &Vec<RequestResponsePair>) -> RequestLogAnalyzerResult {
    let times: Vec<i64> = request_response_pairs.iter()
        .map(|rr: &RequestResponsePair| -> i64 {rr.response.response_time.num_milliseconds() })
        .collect();

    let sum: usize = times.iter().sum::<i64>() as usize;
    let avg: usize = sum / times.len();

    let max: usize = *times.iter().max().unwrap() as usize;
    let min: usize = *times.iter().min().unwrap() as usize;

    let percentile90: usize = percentile(&times, 0.9) as usize;

    let median = median(times.into_iter()).unwrap() as usize;

    RequestLogAnalyzerResult {
        count: request_response_pairs.len().into(),
        max: max,
        min: min,
        avg: avg,
        median: median,
        percentile90: percentile90,
    }
}

fn render_terminal(result: RequestLogAnalyzerResult) {
    println!("count:\t{}", result.count);
    println!("time.avg:\t{}", result.avg);
    println!("time.min:\t{}", result.min);
    println!("time.median:\t{}", result.median);
    println!("time.90percent:\t{}", result.percentile90);
    println!("time.max:\t{}", result.max);
}


fn main() {
    let matches = App::new("Request.log Analyzer")
        .arg(Arg::with_name("filename")
            .index(1)
            .value_name("FILE")
            .help("Log file to analyze")
            .takes_value(true))
        .arg(Arg::with_name("time_filter_minutes")
                .value_name("MINUTES")
                .short("t")
                .help("Limit to the last n minutes")
                .takes_value(true))
                .get_matches();

    let filename = matches.value_of("filename").unwrap();

    let time_filter = match matches.value_of("time_filter_minutes") {
        Some(minutes) => Some(Duration::minutes(minutes.parse().unwrap())),
        None => None
    };

    let lines = open_logfile(filename, time_filter);
    let (requests, responses) = lines.unwrap();

    let pairs: Vec<RequestResponsePair> = pair_requests_responses(requests, responses);

    let result: RequestLogAnalyzerResult = analyze(&pairs);

    render_terminal(result);
}

#[cfg(test)]
mod tests {
	use super::*;
    use request_response::*;
    extern crate time;
    use time::Duration;

    #[test]
    fn test_open_logfile() {
        let lines = open_logfile("src/test/simple-1.log", None);
        let (requests, responses) = lines.unwrap();

        assert_eq!(requests.len(), 2);
        assert_eq!(responses.len(), 2);
    }

    #[test]
    fn test_open_logfile_time_filter() {
        let time_filter: Duration = Duration::minutes(1);
        let lines = open_logfile("src/test/simple-1.log", Some(time_filter));
        let (requests, responses) = lines.unwrap();

        assert_eq!(requests.len(), 0);

        let time_filter: Duration = Duration::minutes(52560000); // 100 years
        let lines = open_logfile("src/test/simple-1.log", Some(time_filter));
        let (requests, responses) = lines.unwrap();

        assert_eq!(requests.len(), 2);
    }

    #[test]
    fn test_pair_requests_resonses() {
        let lines = open_logfile("src/test/simple-1.log", None);
        let (requests, responses) = lines.unwrap();

        let result = pair_requests_responses(requests, responses);

        assert_eq!(result.len(), 2);

        assert_eq!(result[0].request.id, result[0].response.id);
        assert_eq!(result[1].request.id, result[1].response.id);
    }

    #[test]
    fn test_request_log_analyzer_result() {
        let lines = open_logfile("src/test/response-time-calculations.log", None);
        let (requests, responses) = lines.unwrap();

        let request_response_pairs = pair_requests_responses(requests, responses);

        let result: RequestLogAnalyzerResult = analyze(&request_response_pairs);

        let expected = RequestLogAnalyzerResult {
            count: 3,
            max: 100,
            min: 1,
            avg: 37,
            median: 10,
            percentile90: 100,
        };

        assert_eq!(result, expected);
    }

    #[test]
    fn test_90_percentile_calculation() {
        let lines = open_logfile("src/test/percentile.log", None);
        let (requests, responses) = lines.unwrap();

        let request_response_pairs = pair_requests_responses(requests, responses);

        let result: RequestLogAnalyzerResult = analyze(&request_response_pairs);

        assert_eq!(result.percentile90, 9);
    }
}
