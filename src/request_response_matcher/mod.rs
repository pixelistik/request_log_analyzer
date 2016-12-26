//! A module to match and filter Request and Response events

use log_parser::*;

#[derive(Clone)]
#[derive(Debug)]
pub struct RequestResponsePair {
    pub request: log_events::Request,
    pub response: log_events::Response,
}

fn get_matching_response<'a>(request: &log_events::Request, responses: &'a Vec<log_events::Response>) -> Option<&'a log_events::Response> {
    match responses.binary_search_by_key(&request.id, |r| r.id) {
        Ok(index) => Some(&responses[index]),
        Err(_) => None
    }
}

pub fn pair_requests_responses(requests:Vec<log_events::Request>, responses: Vec<log_events::Response>) -> Vec<RequestResponsePair> {
    let mut request_response_pairs: Vec<RequestResponsePair> = Vec::new();

    for request in requests  {
        if let Some(response) = get_matching_response(&request, &responses) {
            request_response_pairs.push(RequestResponsePair{
                request: request.clone(),
                response: response.clone()
            })
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
    fn test_pair_requests_responses() {
        let requests = vec![
            log_parser::log_events::Request {
                id: 1,
                time: DateTime::parse_from_str("08/Apr/2016:09:57:47 +0200", "%d/%b/%Y:%H:%M:%S %z").unwrap(),
                url: "/some/path.html".to_string(),
                original_log_line: "whatever".to_string(),
            },
        ];

        let responses = vec![
            log_parser::log_events::Response {
                id: 1,
                time: DateTime::parse_from_str("08/Apr/2016:09:57:47 +0200", "%d/%b/%Y:%H:%M:%S %z").unwrap(),
                mime_type: "text/html".to_string(),
                response_time: Duration::milliseconds(7),
                http_status: HttpStatus::OK,
                original_log_line: "whatever".to_string(),
            },
            log_parser::log_events::Response {
                id: 99,
                time: DateTime::parse_from_str("08/Apr/2016:09:58:47 +0200", "%d/%b/%Y:%H:%M:%S %z").unwrap(),
                mime_type: "text/html".to_string(),
                response_time: Duration::milliseconds(10),
                http_status: HttpStatus::OK,
                original_log_line: "whatever".to_string(),
            },
        ];

        let result = pair_requests_responses(requests, responses);

        assert_eq!(result.len(), 1);

        assert_eq!(result[0].request.id, 1);
        assert_eq!(result[0].response.id, 1);
    }
}
