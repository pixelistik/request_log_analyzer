//! A module to match and filter Request and Response events

use log_parser::*;

#[derive(Clone)]
#[derive(Debug)]
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

#[cfg(test)]
mod tests {
    use chrono::*;
    use http_status::*;
    use super::*;
    use log_parser;

    #[test]
    fn test_extract_matching_request_response_pairs() {
        let mut requests = vec![log_parser::log_events::Request {
                                    id: 1,
                                    time: DateTime::parse_from_str("08/Apr/2016:09:57:47 +0200",
                                                                   "%d/%b/%Y:%H:%M:%S %z")
                                        .unwrap(),
                                    url: "/some/path.html".to_string(),
                                    original_log_line: "whatever".to_string(),
                                },
                                log_parser::log_events::Request {
                                    id: 77,
                                    time: DateTime::parse_from_str("08/Apr/2016:09:57:47 +0200",
                                                                   "%d/%b/%Y:%H:%M:%S %z")
                                        .unwrap(),
                                    url: "/irrelevant.html".to_string(),
                                    original_log_line: "whatever".to_string(),
                                }];

        let mut responses = vec![log_parser::log_events::Response {
                                     id: 1,
                                     time: DateTime::parse_from_str("08/Apr/2016:09:57:47 +0200",
                                                                    "%d/%b/%Y:%H:%M:%S %z")
                                         .unwrap(),
                                     mime_type: "text/html".to_string(),
                                     response_time: Duration::milliseconds(7),
                                     http_status: HttpStatus::OK,
                                     original_log_line: "whatever".to_string(),
                                 },
                                 log_parser::log_events::Response {
                                     id: 99,
                                     time: DateTime::parse_from_str("08/Apr/2016:09:58:47 +0200",
                                                                    "%d/%b/%Y:%H:%M:%S %z")
                                         .unwrap(),
                                     mime_type: "text/html".to_string(),
                                     response_time: Duration::milliseconds(10),
                                     http_status: HttpStatus::OK,
                                     original_log_line: "whatever".to_string(),
                                 }];

        let result = extract_matching_request_response_pairs(&mut requests, &mut responses);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].request.id, 1);
        assert_eq!(result[0].response.id, 1);

        assert_eq!(requests.len(), 1);
        assert_eq!(responses.len(), 1);
    }
}
