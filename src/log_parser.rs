use std::io;
use std::io::BufRead;
use std::io::Write;
use chrono::*;
use http_status::HttpStatus;

// http://stackoverflow.com/a/27590832/376138
macro_rules! println_stderr(
    ($($arg:tt)*) => { {
        let r = writeln!(&mut ::std::io::stderr(), $($arg)*);
        r.expect("failed printing to stderr");
    } }
);

#[derive(Eq, PartialEq, Clone)]
#[derive(Debug)]
pub struct Request {
    pub id: i32,
    pub time: DateTime<FixedOffset>,
    pub url: String
}

impl Request {
    fn new_from_log_line(log_line: &String) -> Result<Request, &'static str> {
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
            url: url.to_string()
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
}

impl Response {
    fn new_from_log_line(log_line: &String) -> Result<Response, &'static str> {
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
        })
    }
}

pub fn parse(reader: &mut io::Read) -> Result<(Vec<Request>,Vec<Response>), &'static str> {
    let input = io::BufReader::new(reader);

    let mut requests: Vec<Request> = Vec::new();
    let mut responses: Vec<Response> = Vec::new();

    for line in input.lines() {
        let line_value = &line.unwrap();

        if line_value.contains("->") {
            match Request::new_from_log_line(&line_value) {
                Ok(r) => {
                    // if time_filter.is_none() ||
                    //   (time_filter.is_some() && r.is_between_times(UTC::now().with_timezone(&r.time.timezone()) - time_filter.unwrap(), UTC::now().with_timezone(&r.time.timezone()))) {
                        requests.push(r);
                    // }
                },
                Err(err) => println_stderr!("Skipped a line: {}", err)
            }
        }

        if line_value.contains("<-") {
            match Response::new_from_log_line(&line_value) {
                Ok(r) => responses.push(r),
                Err(err) => println_stderr!("Skipped a line: {}", err)
            }
        }

    }

    responses.sort_by_key(|r| r.id);

    Ok((requests, responses))
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use super::*;

    #[test]
    fn test_parse_simple() {
        let mut input_reader = File::open("src/test/simple-1.log").unwrap();

        let (requests, responses) = parse(&mut input_reader).unwrap();

        assert_eq!(requests.len(), 2);
        assert_eq!(responses.len(), 2);
    }
}
