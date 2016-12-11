use stats::median;
use request_response_matcher::*;

pub mod percentile;

#[derive(PartialEq)]
#[derive(Debug)]
pub struct RequestLogAnalyzerResult {
    pub count: usize,
    pub max: usize,
    pub min: usize,
    pub avg: usize,
    pub median: usize,
    pub percentile90: usize,
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

    let percentile90: usize = percentile::percentile(&times, 0.9) as usize;

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

mod tests {
    use log_parser::log_events::*;
    use request_response_matcher::*;
    use super::*;

    #[test]
    fn test_analyze() {
        let request_response_pairs = vec![
            RequestResponsePair {
                request: Request::new_from_log_line(&"08/Apr/2016:09:57:47 +0200 [001] -> GET /content/some/page.html HTTP/1.1".to_string()).unwrap(),
                response: Response::new_from_log_line(&"08/Apr/2016:09:57:47 +0200 [001] <- 200 text/html 1ms".to_string()).unwrap(),
            },
            RequestResponsePair {
                request: Request::new_from_log_line(&"08/Apr/2016:09:58:47 +0200 [02] -> GET /content/some/other.html HTTP/1.1".to_string()).unwrap(),
                response: Response::new_from_log_line(&"08/Apr/2016:09:58:47 +0200 [02] <- 200 text/html 10ms".to_string()).unwrap(),
            },
            RequestResponsePair {
                request: Request::new_from_log_line(&"08/Apr/2016:10:58:47 +0200 [03] -> GET /content/some/third.html HTTP/1.1".to_string()).unwrap(),
                response: Response::new_from_log_line(&"08/Apr/2016:10:58:47 +0200 [03] <- 200 text/html 100ms".to_string()).unwrap(),
            },
        ];

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
    fn test_analyze_empty() {
        let request_response_pairs = vec![];

        let result = analyze(&request_response_pairs);

        let expected = None;

        assert_eq!(result, expected);
    }
}
