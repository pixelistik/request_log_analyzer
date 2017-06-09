use chrono::*;

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum HttpError {
    ClientError4xx,
    ServerError5xx,
}

#[derive(PartialEq,Debug)]
pub enum LogEvent {
    Request(Request),
    Response(Response),
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Request {
    pub id: i32,
    pub time: DateTime<FixedOffset>,
    pub original_log_line: String,
}

impl Request {
    pub fn new_from_log_line(log_line: &String) -> Result<Request, &'static str> {
        let parts: Vec<&str> = log_line.split(' ').collect();

        let id = match parts.get(2) {
            Some(id) => id,
            None => return Err("Uncomprehensible request logline"),
        };

        // Shortest valid id format is "[1]"
        if id.len() < 3 {
            return Err("Uncomprehensible request logline");
        }

        let id_parsed: i32 = match id[1..id.len() - 1].parse() {
            Ok(id) => id,
            Err(_) => return Err("Uncomprehensible request logline"),
        };

        let date = &format!("{} {}", parts[0], parts[1]);

        let date_parsed = match DateTime::parse_from_str(date, "%d/%b/%Y:%H:%M:%S %z") {
            Ok(date_time) => date_time,
            Err(_) => return Err("Uncomprehensible request logline"),
        };

        Ok(Request {
            id: id_parsed,
            time: date_parsed,
            original_log_line: log_line.clone(),
        })
    }
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Response {
    pub id: i32,
    pub response_time: Duration,
    pub original_log_line: String,
    pub http_error: Option<HttpError>,
}

impl Response {
    pub fn new_from_log_line(log_line: &String) -> Result<Response, &'static str> {
        let parts: Vec<&str> = log_line.split(' ').collect();

        let id = parts[2];

        // Shortest valid id format is "[1]"
        if id.len() < 3 {
            return Err("Uncomprehensible response logline");
        }

        let id_numeric: i32 = match id[1..id.len() - 1].parse() {
            Ok(number) => number,
            Err(_) => return Err("Uncomprehensible response logline"),
        };

        let response_time = parts[parts.len() - 1];
        if response_time.len() < 3 {
            return Err("Uncomprehensible response logline");
        }

        let response_time_duration = match response_time[0..response_time.len() - 2].parse() {
            Ok(number) => Duration::milliseconds(number),
            Err(_) => return Err("Uncomprehensible response logline"),
        };

        let http_error = match parts[4].chars().nth(0) {
            Some('4') => Some(HttpError::ClientError4xx),
            Some('5') => Some(HttpError::ServerError5xx),
            _ => None,
        };

        Ok(Response {
            id: id_numeric,
            response_time: response_time_duration,
            http_error: http_error,
            original_log_line: log_line.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use::chrono::*;
    use super::*;

    #[test]
    fn test_parse_request_line() {
        let line = "08/Apr/2016:09:58:47 +0200 [02] -> GET /content/some/other.html HTTP/1.1"
            .to_string();

        let expected = Request {
            id: 2,
            time: DateTime::parse_from_str("08/Apr/2016:09:58:47 +0200", "%d/%b/%Y:%H:%M:%S %z")
                .unwrap(),
            original_log_line: line.clone(),
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
        let line = "99/XYZ/9999:09:99:99 +9900 [02] -> GET /content/some/other.html HTTP/1.1"
            .to_string();

        let expected: Result<Request, &'static str> = Err("Uncomprehensible request logline");
        let result: Result<Request, &'static str> = Request::new_from_log_line(&line);

        assert_eq!(result.is_err(), true);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_request_line_bad_id_format() {
        let line = "08/Apr/2016:09:58:47 +0200 2 -> GET /content/some/other.html HTTP/1.1"
            .to_string();

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
            response_time: Duration::milliseconds(10),
            original_log_line: line.clone(),
            http_error: None,
        };

        let result = Response::new_from_log_line(&line);

        assert_eq!(result.unwrap(), expected)
    }

    #[test]
    fn test_parse_response_line_inconsistent_space() {
        let line = "06/Apr/2016:14:54:16 +0200 [200] <- 200 text/html; charset=utf-8 250ms"
            .to_string();

        let expected = Response {
            id: 200,
            response_time: Duration::milliseconds(250),
            original_log_line: line.clone(),
            http_error: None,
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
    fn test_parse_response_line_client_error() {
        let line = "08/Apr/2016:09:58:48 +0200 [02] <- 400 text/html 10ms".to_string();

        let result = Response::new_from_log_line(&line).unwrap().http_error;
        let expected = Some(HttpError::ClientError4xx);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_response_line_server_error() {
        let line = "08/Apr/2016:09:58:48 +0200 [02] <- 500 text/html 10ms".to_string();

        let result = Response::new_from_log_line(&line).unwrap().http_error;
        let expected = Some(HttpError::ServerError5xx);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_log_event_type() {
        let request_line =
            "08/Apr/2016:09:58:47 +0200 [02] -> GET /content/some/other.html HTTP/1.1".to_string();
        let response_line = "08/Apr/2016:09:58:48 +0200 [02] <- 200 text/html 10ms".to_string();

        let _ = LogEvent::Request(Request::new_from_log_line(&request_line).unwrap());
        let _ = LogEvent::Response(Response::new_from_log_line(&response_line).unwrap());
    }
}
