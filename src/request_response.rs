use chrono::*;

use http_status::HttpStatus;

#[derive(Eq, PartialEq, Clone)]
#[derive(Debug)]
pub struct Request {
    pub id: i32,
    pub time: DateTime<FixedOffset>,
    pub url: String,
    contains_term: Option<bool>,
}

impl Request {
    pub fn new_from_log_line(log_line: &String, check_term: Option<&str>) -> Result<Request, &'static str> {
        let parts: Vec<&str> = log_line.split(" ").collect();

        let id = match parts.get(2) {
            Some(id) =>  id,
            None => return Err("Uncomprehensible request logline")
        };

        // Shortest valid id format is "[1]"
        if id.len() < 3 {
            return Err("Uncomprehensible request logline");
        }

        let id_parsed: i32 = match id[1..id.len()-1].parse() {
            Ok(id) =>  id,
            Err(_) => return Err("Uncomprehensible request logline")
        };

        let url = match parts.get(5) {
            Some(url) =>  url,
            None => return Err("Uncomprehensible request logline")
        };

        let date = &format!("{} {}", parts[0], parts[1]);

        let date_parsed = match DateTime::parse_from_str(date, "%d/%b/%Y:%H:%M:%S %z") {
            Ok(date_time) => date_time,
            Err(_) => return Err("Uncomprehensible request logline")
        };

        Ok(Request {
            id: id_parsed,
            time: date_parsed,
            url: url.to_string(),
            contains_term: match check_term {
                Some(t) => Some(log_line.contains(t)),
                None => None
            },
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
    contains_term: Option<bool>,
}

impl Response {
    pub fn new_from_log_line(log_line: &String, check_term: Option<&str>) -> Result<Response, &'static str> {
        let parts: Vec<&str> = log_line.split(" ").collect();

        let id = parts[2];

        // Shortest valid id format is "[1]"
        if id.len() < 3 {
            return Err("Uncomprehensible response logline");
        }

        let response_time = parts[parts.len()-1];
        if response_time.len() < 3 {
            return Err("Uncomprehensible response logline");
        }

        let response_time_duration = match response_time[0..response_time.len()-2].parse() {
            Ok(number) => Duration::milliseconds(number),
            Err(_) => return Err("Uncomprehensible response logline")
        };

        // Handle special case where the mime type sometimes contains
        // a space, so we need to re-assemble it
        let mime_type = match parts.len() {
            8 => format!("{} {}", parts[5], parts[6]),
            _ => parts[5].to_string()
        };

        let status_code = match parts[4].parse() {
            Ok(number) => HttpStatus::from_code(number),
            Err(_) => return Err("Uncomprehensible response logline")
        };

        Ok(Response {
            id: id[1..id.len()-1].parse().unwrap(),
            time: DateTime::parse_from_str(&format!("{} {}", parts[0], parts[1]), "%d/%b/%Y:%H:%M:%S %z").unwrap(),
            response_time: response_time_duration,
            mime_type: mime_type,
            http_status: status_code,
            contains_term: match check_term {
                Some(t) => Some(log_line.contains(t)),
                None => None
            },
        })
    }
}

pub struct RequestResponsePair {
    pub request: Request,
    pub response: Response
}

impl RequestResponsePair {
    pub fn matches_include_filter(&self) -> bool {
        let term_to_bool = |contains_term|
            match contains_term {
                Some(contains_term) => contains_term,
                None => true
            };

        term_to_bool(self.request.contains_term) || term_to_bool(self.request.contains_term)
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
                contains_term: None,
            },
            Response {
                id: 2,
                time: DateTime::parse_from_str("08/Apr/2016:09:58:47 +0200", "%d/%b/%Y:%H:%M:%S %z").unwrap(),
                mime_type: "text/html".to_string(),
                response_time: Duration::milliseconds(10),
                http_status: HttpStatus::OK,
                contains_term: None,
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
            contains_term: None
        };

        let result = Request::new_from_log_line(&line, None);

        assert_eq!(result.unwrap(), expected)
    }

    #[test]
    fn test_parse_request_line_bad_format() {
        let line = "08/A16:09:58:47 justsomegarbage".to_string();

        let expected: Result<Request, &'static str> = Err("Uncomprehensible request logline");
        let result: Result<Request, &'static str> = Request::new_from_log_line(&line, None);

        assert_eq!(result.is_err(), true);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_request_line_bad_format_but_enough_parts() {
        let line = "just some garbage with more parts at the end".to_string();

        let expected: Result<Request, &'static str> = Err("Uncomprehensible request logline");
        let result: Result<Request, &'static str> = Request::new_from_log_line(&line, None);

        assert_eq!(result.is_err(), true);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_request_line_bad_date_format() {
        let line = "99/XYZ/9999:09:99:99 +9900 [02] -> GET /content/some/other.html HTTP/1.1".to_string();

        let expected: Result<Request, &'static str> = Err("Uncomprehensible request logline");
        let result: Result<Request, &'static str> = Request::new_from_log_line(&line, None);

        assert_eq!(result.is_err(), true);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_request_line_bad_id_format() {
        let line = "08/Apr/2016:09:58:47 +0200 2 -> GET /content/some/other.html HTTP/1.1".to_string();

        let expected: Result<Request, &'static str> = Err("Uncomprehensible request logline");
        let result: Result<Request, &'static str> = Request::new_from_log_line(&line, None);

        assert_eq!(result.is_err(), true);
        assert_eq!(result, expected);
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
            contains_term: None,
        };

        let result = Response::new_from_log_line(&line, None);

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
            contains_term: None,
        };

        let result = Response::new_from_log_line(&line, None);

        assert_eq!(result.unwrap(), expected)
    }

    #[test]
    fn test_parse_response_line_bad_id_format() {
        let line = "08/Apr/2016:09:58:48 +0200 2 <- 200 text/html 10ms".to_string();

        let expected: Result<Response, &'static str> = Err("Uncomprehensible response logline");
        let result: Result<Response, &'static str> = Response::new_from_log_line(&line, None);

        assert_eq!(result.is_err(), true);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_response_line_bad_response_time_too_short() {
        let line = "08/Apr/2016:09:57:47 +0200 [001] <- 200 text/html X".to_string();

        let expected: Result<Response, &'static str> = Err("Uncomprehensible response logline");
        let result: Result<Response, &'static str> = Response::new_from_log_line(&line, None);

        assert_eq!(result.is_err(), true);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_response_line_bad_response_time_not_a_number() {
        let line = "08/Apr/2016:09:57:47 +0200 [001] <- 200 text/html XXXms".to_string();

        let expected: Result<Response, &'static str> = Err("Uncomprehensible response logline");
        let result: Result<Response, &'static str> = Response::new_from_log_line(&line, None);

        assert_eq!(result.is_err(), true);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_response_line_bad_status_code() {
        let line = "08/Apr/2016:09:57:47 +0200 [001] <- FOO text/html 10ms".to_string();

        let expected: Result<Response, &'static str> = Err("Uncomprehensible response logline");
        let result: Result<Response, &'static str> = Response::new_from_log_line(&line, None);

        assert_eq!(result.is_err(), true);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_get_matching_response() {
        let request = Request {
            id: 2,
            time: DateTime::parse_from_str("08/Apr/2016:09:58:47 +0200", "%d/%b/%Y:%H:%M:%S %z").unwrap(),
            url: "/content/some/other.html".to_string(),
            contains_term: None,
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
            contains_term: None,
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
            contains_term: None,
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
            contains_term: None,
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
            contains_term: None,
        };

        let start: DateTime<FixedOffset> = DateTime::parse_from_str("08/Apr/2016:09:00:00 +0200", "%d/%b/%Y:%H:%M:%S %z").unwrap();
        let end: DateTime<FixedOffset> = DateTime::parse_from_str("08/Apr/2016:10:00:00 +0200", "%d/%b/%Y:%H:%M:%S %z").unwrap();

        assert_eq!(request.is_between_times(start, end), true);
    }

    #[test]
    fn test_matches_include_filter_no_term_given() {
        let pair  = RequestResponsePair {
            request: Request::new_from_log_line(
                &"08/Apr/2016:09:57:47 +0200 [001] -> GET /content/some/page.html HTTP/1.1".to_string(),
                None,
            ).unwrap(),
            response: Response::new_from_log_line(
                &"08/Apr/2016:09:57:47 +0200 [001] <- 200 text/html 7ms".to_string(),
                None,
        ).unwrap()
        };

        assert_eq!(pair.matches_include_filter(), true);
    }

    #[test]
    fn test_matches_include_filter_term_found() {
        let pair  = RequestResponsePair {
            request: Request::new_from_log_line(
                &"08/Apr/2016:09:57:47 +0200 [001] -> GET /content/some/page.html HTTP/1.1".to_string(),
                Some(".html")
            ).unwrap(),
            response: Response::new_from_log_line(
                &"08/Apr/2016:09:57:47 +0200 [001] <- 200 text/html 7ms".to_string(),
                Some(".html")
        ).unwrap()
        };

        assert_eq!(pair.matches_include_filter(), true);
    }

    #[test]
    fn test_matches_include_filter_term_given_but_not_found() {
        let pair  = RequestResponsePair {
            request: Request::new_from_log_line(
                &"08/Apr/2016:09:57:47 +0200 [001] -> GET /content/some/page.html HTTP/1.1".to_string(),
                Some("not present term")
            ).unwrap(),
            response: Response::new_from_log_line(
                &"08/Apr/2016:09:57:47 +0200 [001] <- 200 text/html 7ms".to_string(),
                Some("not present term")
        ).unwrap()
        };

        assert_eq!(pair.matches_include_filter(), false);
    }
}
