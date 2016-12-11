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

}
