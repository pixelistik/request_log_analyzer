use request_response_matcher::*;
use chrono::*;

#[derive(Debug)]
#[derive(PartialEq)]
pub struct FilterConditions {
    pub include_terms: Option<Vec<String>>,
    pub exclude_terms: Option<Vec<String>>,
    pub latest_time: Option<Duration>,
}

pub fn matches_filter(pair: &RequestResponsePair, conditions: &FilterConditions) -> bool {
    let matches_include_terms: bool = match conditions.include_terms {
        Some(ref include_terms) => {
            include_terms.iter()
                .fold(false, |result, include_term| {
                    result || pair.request.original_log_line.contains(include_term) ||
                    pair.response.original_log_line.contains(include_term)
                })
        }
        None => true,
    };

    let matches_exclude_terms: bool = match conditions.exclude_terms {
        Some(ref exclude_terms) => {
            !exclude_terms.iter()
                .fold(false, |result, exclude_term| {
                    result || pair.request.original_log_line.contains(exclude_term) ||
                    pair.response.original_log_line.contains(exclude_term)
                })
        }
        None => true,
    };

    let matches_time: bool = match conditions.latest_time {
        Some(latest_time) => {
            let timezone = pair.request.time.timezone();
            let now = UTC::now().with_timezone(&timezone);
            let include_since_time = now - latest_time;
            pair.request.time >= include_since_time
        }
        None => true,
    };

    matches_include_terms && matches_exclude_terms && matches_time
}

#[cfg(test)]
mod tests {
    use log_parser::log_events::*;
    use chrono::*;
    use super::*;

    fn get_fixture() -> RequestResponsePair {
        RequestResponsePair {
            request: Request::new_from_log_line(&"08/Apr/2016:09:57:47 +0200 [001] -> GET \
                                                  /content/some/page.html HTTP/1.1"
                    .to_string())
                .unwrap(),
            response: Response::new_from_log_line(&"08/Apr/2016:09:57:47 +0200 [001] <- 200 \
                                                    text/html 1ms"
                    .to_string())
                .unwrap(),
        }
    }

    #[test]
    fn test_filter_none() {
        let pair = get_fixture();

        let conditions = FilterConditions {
            include_terms: None,
            exclude_terms: None,
            latest_time: None,
        };

        let result = matches_filter(&pair, &conditions);

        assert_eq!(result, true);
    }

    #[test]
    fn test_filter_include_request() {
        let pair = get_fixture();

        let conditions = FilterConditions {
            include_terms: Some(vec!["page.html".to_string()]),
            exclude_terms: None,
            latest_time: None,
        };

        let result = matches_filter(&pair, &conditions);

        assert_eq!(result, true);
    }

    #[test]
    fn test_filter_include_response() {
        let pair = get_fixture();

        let conditions = FilterConditions {
            include_terms: Some(vec!["text/html".to_string()]),
            exclude_terms: None,
            latest_time: None,
        };

        let result = matches_filter(&pair, &conditions);

        assert_eq!(result, true);
    }

    #[test]
    fn test_filter_exclude_request() {
        let pair = get_fixture();

        let conditions = FilterConditions {
            include_terms: None,
            exclude_terms: Some(vec!["page.html".to_string()]),
            latest_time: None,
        };

        let result = matches_filter(&pair, &conditions);

        assert_eq!(result, false);
    }

    #[test]
    fn test_filter_exclude_response() {
        let pair = get_fixture();

        let conditions = FilterConditions {
            include_terms: None,
            exclude_terms: Some(vec!["text/html".to_string()]),
            latest_time: None,
        };

        let result = matches_filter(&pair, &conditions);

        assert_eq!(result, false);
    }

    #[test]
    fn test_filter_include_multiple() {
        let pair = get_fixture();

        let conditions = FilterConditions {
            include_terms: Some(vec!["irrelevant.html".to_string(), "page.html".to_string()]),
            exclude_terms: None,
            latest_time: None,
        };

        let result = matches_filter(&pair, &conditions);

        assert_eq!(result, true);
    }

    #[test]
    fn test_filter_exclude_multiple() {
        let pair = get_fixture();

        let conditions = FilterConditions {
            include_terms: None,
            exclude_terms: Some(vec!["irrelevant.html".to_string(), "page.html".to_string()]),
            latest_time: None,
        };

        let result = matches_filter(&pair, &conditions);

        assert_eq!(result, false);
    }

    #[test]
    fn test_filter_time_matches() {
        let mut pair = get_fixture();
        pair.request.time = UTC::now().with_timezone(&pair.request.time.timezone());

        let conditions = FilterConditions {
            include_terms: None,
            exclude_terms: None,
            latest_time: Some(Duration::minutes(10)),
        };

        let result = matches_filter(&pair, &conditions);

        assert_eq!(result, true);
    }

    #[test]
    fn test_filter_time_matches_not() {
        let mut pair = get_fixture();
        pair.request.time = UTC::now().with_timezone(&pair.request.time.timezone()) -
                            Duration::minutes(12);

        let conditions = FilterConditions {
            include_terms: None,
            exclude_terms: None,
            latest_time: Some(Duration::minutes(10)),
        };

        let result = matches_filter(&pair, &conditions);

        assert_eq!(result, false);
    }
}
