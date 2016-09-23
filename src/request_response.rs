use std::io;
use chrono::*;

use http_status::HttpStatus;

#[derive(Eq, PartialEq, Clone)]
#[derive(Debug)]
pub struct Request {
    pub id: i32,
    pub time: DateTime<FixedOffset>,
    pub url: String,
    original_log_line: String,
}

impl Request {
    pub fn new_from_log_line(log_line: &String) -> Result<Request, io::Error> {
        let parts: Vec<&str> = log_line.split(" ").collect();


        let id = parts[2];
        let url = parts[5];

        Ok(Request {
            id: id[1..id.len()-1].parse().unwrap(),
            time: DateTime::parse_from_str(&format!("{} {}", parts[0], parts[1]), "%d/%b/%Y:%H:%M:%S %z").unwrap(),
            url: url.to_string(),
            original_log_line: log_line.clone(),
        })
    }

    pub fn get_matching_response<'a>(&'a self, responses: &'a Vec<Response>) -> Option<&Response> {
        match responses.binary_search_by_key(&self.id, |r| r.id) {
            Ok(index) => Some(&responses[index]),
            Err(_) => None
        }
    }

    pub fn is_between_times(&self, start: DateTime<FixedOffset>, end: DateTime<FixedOffset>) -> bool {
        start < self.time && self.time <= end
    }


}

#[derive(Eq, PartialEq, Clone)]
#[derive(Debug)]
pub struct Response {
    pub id: i32,
    pub time: DateTime<FixedOffset>,
    pub mime_type: String,
    pub response_time: Duration,
    pub http_status: HttpStatus,
    original_log_line: String,
}

impl Response {
    pub fn new_from_log_line(log_line: &String) -> Result<Response, io::Error> {
        let parts: Vec<&str> = log_line.split(" ").collect();

        let id = parts[2];
        let response_time = parts[parts.len()-1];

        // Handle special case where the mime type sometimes contains
        // a space, so we need to re-assemble it
        let mime_type = match parts.len() {
            8 => format!("{} {}", parts[5], parts[6]),
            _ => parts[5].to_string()
        };

        let status_code = parts[4];

        Ok(Response {
            id: id[1..id.len()-1].parse().unwrap(),
            time: DateTime::parse_from_str(&format!("{} {}", parts[0], parts[1]), "%d/%b/%Y:%H:%M:%S %z").unwrap(),
            response_time: Duration::milliseconds(response_time[0..response_time.len()-2].parse().unwrap()),
            mime_type: mime_type,
            http_status: HttpStatus::from_code(status_code.parse().unwrap()),
            original_log_line: log_line.clone(),
        })
    }
}

pub struct RequestResponsePair {
    pub request: Request,
    pub response: Response
}

impl RequestResponsePair {
    pub fn matches_include_exclude_filter(&self, include_term: Option<&str>, exclude_term: Option<&str>) -> bool {
        let include_result = match include_term {
            Some(include_term) => self.request.original_log_line.contains(include_term) || self.response.original_log_line.contains(include_term),
            None => true
        };

        let exclude_result = match exclude_term {
            Some(exclude_term) => !self.request.original_log_line.contains(exclude_term) && !self.response.original_log_line.contains(exclude_term),
            None => true
        };

        include_result && exclude_result
    }
}

pub fn pair_requests_responses(requests:Vec<Request>, responses: Vec<Response>) -> Vec<RequestResponsePair> {
    let mut request_response_pairs: Vec<RequestResponsePair> = Vec::new();

    for request in requests  {
        if let Some(response) = request.get_matching_response(&responses) {
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
	use super::*;
    use::chrono::*;
    use http_status::HttpStatus;

    fn get_simple_responses_fixture() -> Vec<Response> {
        vec![
            Response {
                id: 1,
                time: DateTime::parse_from_str("08/Apr/2016:09:57:47 +0200", "%d/%b/%Y:%H:%M:%S %z").unwrap(),
                mime_type: "text/html".to_string(),
                response_time: Duration::milliseconds(7),
                http_status: HttpStatus::OK,
                original_log_line: "foo".to_string(),
            },
            Response {
                id: 2,
                time: DateTime::parse_from_str("08/Apr/2016:09:58:47 +0200", "%d/%b/%Y:%H:%M:%S %z").unwrap(),
                mime_type: "text/html".to_string(),
                response_time: Duration::milliseconds(10),
                http_status: HttpStatus::OK,
                original_log_line: "foo".to_string( ),
            },
        ]
    }

    #[test]
    fn test_parse_request_line() {
        let line = "08/Apr/2016:09:58:47 +0200 [02] -> GET /content/some/other.html HTTP/1.1".to_string();

        let expected = Request {
            id: 2,
            time: DateTime::parse_from_str("08/Apr/2016:09:58:47 +0200", "%d/%b/%Y:%H:%M:%S %z").unwrap(),
            url: "/content/some/other.html".to_string(),
            original_log_line: "08/Apr/2016:09:58:47 +0200 [02] -> GET /content/some/other.html HTTP/1.1".to_string(),
        };

        let result = Request::new_from_log_line(&line);

        assert_eq!(result.unwrap(), expected)
    }

    #[test]
    fn test_parse_response_line() {
        let line = "08/Apr/2016:09:58:48 +0200 [02] <- 200 text/html 10ms".to_string();

        let expected = Response {
            id: 2,
            time: DateTime::parse_from_str("08/Apr/2016:09:58:48 +0200", "%d/%b/%Y:%H:%M:%S %z").unwrap(),
            mime_type: "text/html".to_string(),
            response_time: Duration::milliseconds(10),
            http_status: HttpStatus::OK,
            original_log_line: "08/Apr/2016:09:58:48 +0200 [02] <- 200 text/html 10ms".to_string(),
        };

        let result = Response::new_from_log_line(&line);

        assert_eq!(result.unwrap(), expected)
    }

    #[test]
    fn test_parse_response_line_inconsistent_space() {
        let line = "06/Apr/2016:14:54:16 +0200 [200] <- 200 text/html; charset=utf-8 250ms".to_string();

        let expected = Response {
            id: 200,
            time: DateTime::parse_from_str("06/Apr/2016:14:54:16 +0200", "%d/%b/%Y:%H:%M:%S %z").unwrap(),
            mime_type: "text/html; charset=utf-8".to_string(),
            response_time: Duration::milliseconds(250),
            http_status: HttpStatus::OK,
            original_log_line: "06/Apr/2016:14:54:16 +0200 [200] <- 200 text/html; charset=utf-8 250ms".to_string(),
        };

        let result = Response::new_from_log_line(&line);

        assert_eq!(result.unwrap(), expected)
    }

    #[test]
    fn test_get_matching_response() {
        let request = Request {
            id: 2,
            time: DateTime::parse_from_str("08/Apr/2016:09:58:47 +0200", "%d/%b/%Y:%H:%M:%S %z").unwrap(),
            url: "/content/some/other.html".to_string(),
            original_log_line: "foo".to_string(),
        };

        let responses = get_simple_responses_fixture();

        let result = request.get_matching_response(&responses);
        assert_eq!(result.unwrap().id, 2);
    }

    #[test]
    fn test_get_matching_response_none_found() {
        let responses = get_simple_responses_fixture();

        let request_without_matching = Request {
            id: 999,
            time: DateTime::parse_from_str("08/Apr/2016:09:58:47 +0200", "%d/%b/%Y:%H:%M:%S %z").unwrap(),
            url: "/content/some/other.html".to_string(),
            original_log_line: "foo".to_string(),
        };

        let result = request_without_matching.get_matching_response(&responses);

        assert!(result.is_none());
    }

    #[test]
    fn test_is_between_times() {
        let request = Request {
            id: 1,
            time: DateTime::parse_from_str("08/Apr/2016:10:00:00 +0200", "%d/%b/%Y:%H:%M:%S %z").unwrap(),
            url: "/content/some/other.html".to_string(),
            original_log_line: "foo".to_string(),
        };

        let start: DateTime<FixedOffset> = DateTime::parse_from_str("08/Apr/2016:09:00:00 +0200", "%d/%b/%Y:%H:%M:%S %z").unwrap();
        let end: DateTime<FixedOffset> = DateTime::parse_from_str("08/Apr/2016:11:00:00 +0200", "%d/%b/%Y:%H:%M:%S %z").unwrap();

        assert_eq!(request.is_between_times(start, end), true);
    }

    #[test]
    fn test_is_between_times_not() {
        let request = Request {
            id: 1,
            time: DateTime::parse_from_str("08/Apr/2016:11:00:00 +0200", "%d/%b/%Y:%H:%M:%S %z").unwrap(),
            url: "/content/some/other.html".to_string(),
            original_log_line: "foo".to_string(),
        };

        let start: DateTime<FixedOffset> = DateTime::parse_from_str("08/Apr/2016:09:00:00 +0200", "%d/%b/%Y:%H:%M:%S %z").unwrap();
        let end: DateTime<FixedOffset> = DateTime::parse_from_str("08/Apr/2016:10:00:00 +0200", "%d/%b/%Y:%H:%M:%S %z").unwrap();

        assert_eq!(request.is_between_times(start, end), false);
    }

    #[test]
    fn test_is_between_times_include_end() {
        let request = Request {
            id: 1,
            time: DateTime::parse_from_str("08/Apr/2016:10:00:00 +0200", "%d/%b/%Y:%H:%M:%S %z").unwrap(),
            url: "/content/some/other.html".to_string(),
            original_log_line: "foo".to_string(),
        };

        let start: DateTime<FixedOffset> = DateTime::parse_from_str("08/Apr/2016:09:00:00 +0200", "%d/%b/%Y:%H:%M:%S %z").unwrap();
        let end: DateTime<FixedOffset> = DateTime::parse_from_str("08/Apr/2016:10:00:00 +0200", "%d/%b/%Y:%H:%M:%S %z").unwrap();

        assert_eq!(request.is_between_times(start, end), true);
    }

    #[test]
    fn test_matches_include_exclude_filter() {
        let pair  = RequestResponsePair {
            request: Request::new_from_log_line(&"08/Apr/2016:09:57:47 +0200 [001] -> GET /content/some/page.html HTTP/1.1".to_string()).unwrap(),
            response: Response::new_from_log_line(&"08/Apr/2016:09:57:47 +0200 [001] <- 200 text/html 7ms".to_string()).unwrap()
        };

        // Include if include term is found
        assert_eq!(pair.matches_include_exclude_filter(Some("page.html"), None), true);
        // Do not include if include term is not found
        assert_eq!(pair.matches_include_exclude_filter(Some("notpresent.html"), None), false);

        // Exclude if exclude term is found
        assert_eq!(pair.matches_include_exclude_filter(None, Some("page.html")), false);
        // Do not exclude if exclude term is not found
        assert_eq!(pair.matches_include_exclude_filter(None, Some("notpresent.html")), true);

        // Exclude if exclude term is found, even is include term is also found
        assert_eq!(pair.matches_include_exclude_filter(Some("page.html"), Some("page.html")), false);
        // Do not include if include term is not found, even if exclude term is also not found
        assert_eq!(pair.matches_include_exclude_filter(Some("notpresent.html"), Some("notpresent.html")), false);
    }
}
