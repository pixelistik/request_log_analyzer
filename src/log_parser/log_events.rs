//! A module to parse invidual log lines into Requests and Responses

use chrono::*;
use http_status::HttpStatus;

#[derive(Eq, PartialEq, Clone)]
#[derive(Debug)]
pub struct Request {
    pub id: i32,
    pub time: DateTime<FixedOffset>,
    pub url: String,
    pub original_log_line: String
}

impl Request {
    pub fn new_from_log_line(log_line: &String) -> Result<Request, &'static str> {
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
            original_log_line: log_line.clone()
        })
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
    pub original_log_line: String,
}

impl Response {
    pub fn new_from_log_line(log_line: &String) -> Result<Response, &'static str> {
        let parts: Vec<&str> = log_line.split(" ").collect();

        let id = parts[2];

        // Shortest valid id format is "[1]"
        if id.len() < 3 {
            return Err("Uncomprehensible response logline");
        }

        let id_numeric: i32 = match id[1..id.len()-1].parse() {
            Ok(number) => number,
            Err(_) => return Err("Uncomprehensible response logline")
        };

        let time = match DateTime::parse_from_str(&format!("{} {}", parts[0], parts[1]), "%d/%b/%Y:%H:%M:%S %z") {
            Ok(time) => time,
            Err(_) => return Err("Uncomprehensible response logline")
        };

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
            id: id_numeric,
            time: time,
            response_time: response_time_duration,
            mime_type: mime_type,
            http_status: status_code,
            original_log_line: log_line.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use::chrono::*;
    use http_status::HttpStatus;

    use super::*;

    #[test]
    fn test_parse_request_line() {
        let line = "08/Apr/2016:09:58:47 +0200 [02] -> GET /content/some/other.html HTTP/1.1".to_string();

        let expected = Request {
            id: 2,
            time: DateTime::parse_from_str("08/Apr/2016:09:58:47 +0200", "%d/%b/%Y:%H:%M:%S %z").unwrap(),
            url: "/content/some/other.html".to_string(),
            original_log_line: line.clone()
        };

        let result = Request::new_from_log_line(&line);

        assert_eq!(result.unwrap(), expected)
    }

    #[test]
    fn test_parse_request_line_bad_format() {
        let line = "08/A16:09:58:47 justsomegarbage".to_string();

        let expected: Result<Request, &'static str> = Err("Uncomprehensible request logline");
        let result: Result<Request, &'static str> = Request::new_from_log_line(&line);

        assert_eq!(result.is_err(), true);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_request_line_bad_format_but_enough_parts() {
        let line = "just some garbage with more parts at the end".to_string();

        let expected: Result<Request, &'static str> = Err("Uncomprehensible request logline");
        let result: Result<Request, &'static str> = Request::new_from_log_line(&line);

        assert_eq!(result.is_err(), true);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_request_line_bad_date_format() {
        let line = "99/XYZ/9999:09:99:99 +9900 [02] -> GET /content/some/other.html HTTP/1.1".to_string();

        let expected: Result<Request, &'static str> = Err("Uncomprehensible request logline");
        let result: Result<Request, &'static str> = Request::new_from_log_line(&line);

        assert_eq!(result.is_err(), true);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_request_line_bad_id_format() {
        let line = "08/Apr/2016:09:58:47 +0200 2 -> GET /content/some/other.html HTTP/1.1".to_string();

        let expected: Result<Request, &'static str> = Err("Uncomprehensible request logline");
        let result: Result<Request, &'static str> = Request::new_from_log_line(&line);

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
            original_log_line: line.clone(),
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
            original_log_line: line.clone(),
        };

        let result = Response::new_from_log_line(&line);

        assert_eq!(result.unwrap(), expected)
    }

    #[test]
    fn test_parse_response_line_bad_id_format() {
        let line = "08/Apr/2016:09:58:48 +0200 2 <- 200 text/html 10ms".to_string();

        let expected: Result<Response, &'static str> = Err("Uncomprehensible response logline");
        let result: Result<Response, &'static str> = Response::new_from_log_line(&line);

        assert_eq!(result.is_err(), true);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_response_line_bad_id_format_no_number() {
        let line = "08/Apr/2016:09:58:48 +0200 [XXX] <- 200 text/html 10ms".to_string();

        let expected: Result<Response, &'static str> = Err("Uncomprehensible response logline");
        let result: Result<Response, &'static str> = Response::new_from_log_line(&line);

        assert_eq!(result.is_err(), true);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_response_line_bad_time_format() {
        let line = "08/Apr/2016:5:2X:48 +0200 [83940] <- 200 text/html 14642ms".to_string();

        let expected: Result<Response, &'static str> = Err("Uncomprehensible response logline");
        let result: Result<Response, &'static str> = Response::new_from_log_line(&line);

        assert_eq!(result.is_err(), true);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_response_line_bad_response_time_too_short() {
        let line = "08/Apr/2016:09:57:47 +0200 [001] <- 200 text/html X".to_string();

        let expected: Result<Response, &'static str> = Err("Uncomprehensible response logline");
        let result: Result<Response, &'static str> = Response::new_from_log_line(&line);

        assert_eq!(result.is_err(), true);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_response_line_bad_response_time_not_a_number() {
        let line = "08/Apr/2016:09:57:47 +0200 [001] <- 200 text/html XXXms".to_string();

        let expected: Result<Response, &'static str> = Err("Uncomprehensible response logline");
        let result: Result<Response, &'static str> = Response::new_from_log_line(&line);

        assert_eq!(result.is_err(), true);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_response_line_bad_status_code() {
        let line = "08/Apr/2016:09:57:47 +0200 [001] <- FOO text/html 10ms".to_string();

        let expected: Result<Response, &'static str> = Err("Uncomprehensible response logline");
        let result: Result<Response, &'static str> = Response::new_from_log_line(&line);

        assert_eq!(result.is_err(), true);
        assert_eq!(result, expected);
    }
}