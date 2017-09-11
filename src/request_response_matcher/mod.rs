use log_parser::*;
use timing_analyzer::Timing;
use error_analyzer::HttpErrorState;

#[derive(Clone, Debug, PartialEq)]
pub struct RequestResponsePair {
    pub request: log_events::Request,
    pub response: log_events::Response,
}

pub fn extract_matching_request_response_pairs(requests: &mut Vec<log_events::Request>,
                                               responses: &mut Vec<log_events::Response>)
                                               -> Vec<RequestResponsePair> {
    let mut request_response_pairs: Vec<RequestResponsePair> = Vec::new();

    for request_index in 0..requests.len() {
        {
            let request = requests.get(request_index);

            if request.is_none() {
                continue;
            }
        }

        let matching_response_index: Option<usize> = responses.iter()
            .position(|response| requests[request_index].id == response.id);

        if matching_response_index.is_some() {
            let request = requests.remove(request_index);
            let response = responses.remove(matching_response_index.unwrap());

            let pair = RequestResponsePair {
                request: request,
                response: response,
            };

            request_response_pairs.push(pair);
        }
    }

    request_response_pairs
}

impl Timing for RequestResponsePair {
    fn num_milliseconds(&self) -> i64 {
        self.response.response_time.num_milliseconds()
    }
}

impl Timing for Box<Timing> {
    fn num_milliseconds(&self) -> i64 {
        (**self).num_milliseconds()
    }
}

impl Timing for Box<RequestResponsePair> {
    fn num_milliseconds(&self) -> i64 {
        (**self).num_milliseconds()
    }
}

impl HttpErrorState for RequestResponsePair {
    fn error(&self) -> Option<log_events::HttpError> {
        self.response.http_error.clone()
    }
}

impl HttpErrorState for Box<RequestResponsePair> {
    fn error(&self) -> Option<log_events::HttpError> {
        (**self).error()
    }
}

pub struct RequestResponsePairIterator<'a> {
    events: &'a mut Iterator<Item = log_events::LogEvent>,
    requests: Vec<log_events::Request>,
    responses: Vec<log_events::Response>,
}

impl<'a> RequestResponsePairIterator<'a> {
    pub fn new(events: &'a mut Iterator<Item = log_events::LogEvent>) -> Self {
        RequestResponsePairIterator {
            events: events,
            requests: vec![],
            responses: vec![],
        }
    }
}

impl<'a> Iterator for RequestResponsePairIterator<'a> {
    type Item = RequestResponsePair;

    fn next(&mut self) -> Option<RequestResponsePair> {
        let mut pair = None;

        while pair.is_none() {
            let event = self.events.next();

            match event {
                Some(log_events::LogEvent::Request(request)) => self.requests.push(request),
                Some(log_events::LogEvent::Response(response)) => self.responses.push(response),
                None => return None,
            }

            pair = extract_first_matching_request_response_pair(&mut self.requests,
                                                                &mut self.responses);
        }
        pair
    }
}

pub fn extract_first_matching_request_response_pair(requests: &mut Vec<log_events::Request>,
                                                    responses: &mut Vec<log_events::Response>)
                                                    -> Option<RequestResponsePair> {
    for request_index in 0..requests.len() {
        {
            let request = requests.get(request_index);

            if request.is_none() {
                continue;
            }
        }

        let matching_response_index: Option<usize> = responses.iter()
            .position(|response| requests[request_index].id == response.id);

        if matching_response_index.is_some() {
            let request = requests.remove(request_index);
            let response = responses.remove(matching_response_index.unwrap());

            let pair = RequestResponsePair {
                request: request,
                response: response,
            };

            return Some(pair);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use chrono::*;
    use super::*;
    use log_parser;
    use timing_analyzer::Timing;
    use error_analyzer::HttpErrorState;

    #[test]
    fn test_extract_matching_request_response_pairs() {
        let mut requests = vec![log_parser::log_events::Request {
                                    id: 1,
                                    time: DateTime::parse_from_str("08/Apr/2016:09:57:47 +0200",
                                                                   "%d/%b/%Y:%H:%M:%S %z")
                                        .unwrap(),
                                    original_log_line: "whatever".to_string(),
                                },
                                log_parser::log_events::Request {
                                    id: 77,
                                    time: DateTime::parse_from_str("08/Apr/2016:09:57:47 +0200",
                                                                   "%d/%b/%Y:%H:%M:%S %z")
                                        .unwrap(),
                                    original_log_line: "whatever".to_string(),
                                }];

        let mut responses = vec![log_parser::log_events::Response {
                                     id: 1,
                                     response_time: Duration::milliseconds(7),
                                     original_log_line: "whatever".to_string(),
                                     http_error: None,
                                 },
                                 log_parser::log_events::Response {
                                     id: 99,
                                     response_time: Duration::milliseconds(10),
                                     original_log_line: "whatever".to_string(),
                                     http_error: None,
                                 }];

        let result = extract_matching_request_response_pairs(&mut requests, &mut responses);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].request.id, 1);
        assert_eq!(result[0].response.id, 1);

        assert_eq!(requests.len(), 1);
        assert_eq!(responses.len(), 1);
    }

    #[test]
    fn test_extract_matching_request_response_pairs_iterator() {
        let events =
            vec![log_parser::log_events::LogEvent::Request(log_parser::log_events::Request {
                     id: 1,
                     time: DateTime::parse_from_str("08/Apr/2016:09:57:47 +0200",
                                                    "%d/%b/%Y:%H:%M:%S %z")
                         .unwrap(),
                     original_log_line: "whatever".to_string(),
                 }),
                 log_parser::log_events::LogEvent::Response(log_parser::log_events::Response {
                     id: 1,
                     response_time: Duration::milliseconds(7),
                     original_log_line: "whatever".to_string(),
                     http_error: None,
                 })];

        let mut events_iter = events.into_iter();
        let mut iterator = RequestResponsePairIterator::new(&mut events_iter);

        let result = iterator.next().unwrap();
        assert_eq!(result.request.id, 1);
    }

    #[test]
    fn test_timing_trait() {
        let timing: &Timing = &RequestResponsePair {
            request: log_parser::log_events::Request {
                id: 1,
                time: DateTime::parse_from_str("08/Apr/2016:09:57:47 +0200",
                                               "%d/%b/%Y:%H:%M:%S %z")
                    .unwrap(),
                original_log_line: "whatever".to_string(),
            },
            response: log_parser::log_events::Response {
                id: 1,
                response_time: Duration::milliseconds(7),
                original_log_line: "whatever".to_string(),
                http_error: None,
            },
        } as &Timing;

        let result: i64 = timing.num_milliseconds();
        assert_eq!(result, 7);

        let boxed_timing = Box::new(timing);

        let result: i64 = boxed_timing.num_milliseconds();
        assert_eq!(result, 7);
    }

    #[test]
    fn test_http_error_state_trait() {
        let state: &HttpErrorState = &RequestResponsePair {
            request: log_parser::log_events::Request {
                id: 1,
                time: DateTime::parse_from_str("08/Apr/2016:09:57:47 +0200",
                                               "%d/%b/%Y:%H:%M:%S %z")
                    .unwrap(),
                original_log_line: "whatever".to_string(),
            },
            response: log_parser::log_events::Response {
                id: 1,
                response_time: Duration::milliseconds(7),
                original_log_line: "whatever".to_string(),
                http_error: None,
            },
        } as &HttpErrorState;

        let result = state.error();
        assert_eq!(result, None);

        let boxed_state = Box::new(state);

        let result = boxed_state.error();
        assert_eq!(result, None);
    }
}
