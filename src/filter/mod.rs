use log_parser::log_events::*;
use request_response_matcher::*;
use chrono::*;

pub struct FilterConditions {
    pub include_terms: Option<Vec<String>>,
    pub exclude_terms: Option<Vec<String>>,
    pub latest_time: Option<Duration>,
}

fn matches_filter(pair: &RequestResponsePair, conditions: &FilterConditions) -> bool {
    let matches_include_terms: bool = match conditions.include_terms {
        Some(ref include_terms) => {
            include_terms.iter()
                .fold(
                    false,
                    |result, include_term|
                        result ||
                        pair.request.original_log_line.contains(include_term) ||
                        pair.response.original_log_line.contains(include_term)
                )
        },
        None => true
    };

    let matches_exclude_terms: bool = match conditions.exclude_terms {
        Some(ref exclude_terms) => {
            !exclude_terms.iter()
                .fold(
                    false,
                    |result, exclude_term|
                        result ||
                        pair.request.original_log_line.contains(exclude_term) ||
                        pair.response.original_log_line.contains(exclude_term)
                )
        },
        None => true
    };

    let matches_time: bool = match conditions.latest_time {
        Some(latest_time) => {
            let timezone = pair.request.time.timezone();
            let now = UTC::now().with_timezone(&timezone);
            let include_since_time = now - latest_time;
            pair.request.time >= include_since_time
        },
        None => true
    };

    matches_include_terms && matches_exclude_terms && matches_time
}

pub fn filter(pairs: &Vec<RequestResponsePair>, conditions: FilterConditions) -> Vec<RequestResponsePair> {
    let filtered_pairs: Vec<RequestResponsePair> = pairs.clone().into_iter()
        .filter(|pair| matches_filter(&pair, &conditions))
        .collect();

    filtered_pairs
}

mod tests {
    use log_parser::log_events::*;
    use request_response_matcher::*;
    use chrono::*;
    use super::*;

    fn get_fixture() -> Vec<RequestResponsePair> {
        vec![
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
        ]
    }

    #[test]
    fn test_filter_none() {
        let request_response_pairs = get_fixture();

        let conditions = FilterConditions {
            include_terms: None,
            exclude_terms: None,
            latest_time: None,
        };

        let result: Vec<RequestResponsePair> = filter(&request_response_pairs, conditions);

        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_filter_include_exclude() {
        let request_response_pairs = get_fixture();

        let conditions = FilterConditions {
            include_terms: Some(vec!["text/html".to_string()]),
            exclude_terms: Some(vec!["third.html".to_string()]),
            latest_time: None,
        };

        let result: Vec<RequestResponsePair> = filter(&request_response_pairs, conditions);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].request.id, 1);
        assert_eq!(result[1].request.id, 2);
    }

    #[test]
    fn test_filter_include_multiple() {
        let request_response_pairs = get_fixture();

        let conditions = FilterConditions {
            include_terms: Some(vec!["page.html".to_string(), "third.html".to_string()]),
            exclude_terms: None,
            latest_time: None,
        };

        let result: Vec<RequestResponsePair> = filter(&request_response_pairs, conditions);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].request.id, 1);
        assert_eq!(result[1].request.id, 3);
    }

    #[test]
    fn test_filter_exclude_multiple() {
        let request_response_pairs = get_fixture();

        let conditions = FilterConditions {
            include_terms: None,
            exclude_terms: Some(vec!["page.html".to_string(), "third.html".to_string()]),
            latest_time: None,
        };

        let result: Vec<RequestResponsePair> = filter(&request_response_pairs, conditions);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].request.id, 2);
    }

    #[test]
    fn test_filter_time() {
        let mut request_response_pairs = get_fixture();
        request_response_pairs[0].request.time = UTC::now().with_timezone(&request_response_pairs[0].request.time.timezone());
        request_response_pairs[1].request.time = UTC::now().with_timezone(&request_response_pairs[1].request.time.timezone()) - Duration::minutes(12);

        let conditions = FilterConditions {
            include_terms: None,
            exclude_terms: None,
            latest_time: Some(Duration::minutes(10)),
        };

        let result: Vec<RequestResponsePair> = filter(&request_response_pairs, conditions);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].request.id, 1);
    }
}
