use std::io::{self, BufReader};
use std::io::BufRead;
use std::fs::File;
extern crate time;
use time::Tm;
use time::strptime;
use time::Duration;
extern crate stats;
use stats::median;
extern crate clap;
use clap::{Arg, App};

mod percentile;
use percentile::percentile;

mod http_status;
use http_status::HttpStatus;

mod request_response;
use request_response::*;

pub fn open_logfile(path: &str) -> Result<(Vec<Request>,Vec<Response>), io::Error> {
    let f = try!(File::open(path));

    let f = BufReader::new(f);

    let mut requests: Vec<Request> = Vec::new();
    let mut responses: Vec<Response> = Vec::new();


    for line in f.lines() {
        let line_value = &line.unwrap();

        if line_value.contains("->") {
            let r = try!(Request::new_from_log_line(&line_value));
            requests.push(r)
        }

        if line_value.contains("<-") {
            let r = try!(Response::new_from_log_line(&line_value));
            responses.push(r)
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
            .get_matches();

    let filename = matches.value_of("filename").unwrap();

    let lines = open_logfile(filename);
    let (requests, responses) = lines.unwrap();

    let pairs: Vec<RequestResponsePair> = pair_requests_responses(requests, responses);

    let result: RequestLogAnalyzerResult = analyze(&pairs);

    render_terminal(result);
}

#[cfg(test)]
mod tests {
	use super::*;
    extern crate time;
    use time::strptime;
    use::time::Duration;
    use http_status::HttpStatus;
    use request_response::*;

    #[test]
    fn test_open_logfile() {
        let lines = open_logfile("src/test/simple-1.log");
        let (requests, responses) = lines.unwrap();

        assert_eq!(requests.len(), 2);
        assert_eq!(responses.len(), 2);
    }

    #[test]
    fn test_get_matching_response() {
        let lines = open_logfile("src/test/simple-1.log");
        let (requests, responses) = lines.unwrap();

        let result = requests[0].get_matching_response(&responses);

        let expected = Response {
            id: 1,
            time: strptime("08/Apr/2016:09:57:47 +0200", "%d/%b/%Y:%H:%M:%S").unwrap(),
            mime_type: "text/html".to_string(),
            response_time: Duration::milliseconds(7),
            http_status: HttpStatus::OK,
        };

        assert_eq!(*result.unwrap(), expected);
    }

    #[test]
    fn test_get_matching_response_none_found() {
        let lines = open_logfile("src/test/simple-1.log");
        let (_, responses) = lines.unwrap();

        let request_without_matching = Request {
            id: 999,
            time: strptime("08/Apr/2016:09:58:47 +0200", "%d/%b/%Y:%H:%M:%S").unwrap(),
            url: "/content/some/other.html".to_string()
        };

        let result = request_without_matching.get_matching_response(&responses);

        assert!(result.is_none());
    }

    #[test]
    fn test_pair_requests_resonses() {
        let lines = open_logfile("src/test/simple-1.log");
        let (requests, responses) = lines.unwrap();

        let result = pair_requests_responses(requests, responses);

        assert_eq!(result.len(), 2);

        assert_eq!(result[0].request.id, result[0].response.id);
        assert_eq!(result[1].request.id, result[1].response.id);
    }

    #[test]
    fn test_request_log_analyzer_result() {
        let lines = open_logfile("src/test/response-time-calculations.log");
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
        let lines = open_logfile("src/test/percentile.log");
        let (requests, responses) = lines.unwrap();

        let request_response_pairs = pair_requests_responses(requests, responses);

        let result: RequestLogAnalyzerResult = analyze(&request_response_pairs);

        assert_eq!(result.percentile90, 9);
    }
}
