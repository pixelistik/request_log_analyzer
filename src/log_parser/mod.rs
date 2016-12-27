use std::io;
use std::io::BufRead;
use std::io::Write;

pub mod log_events;
use self::log_events::*;

// http://stackoverflow.com/a/27590832/376138
macro_rules! println_stderr(
    ($($arg:tt)*) => { {
        let r = writeln!(&mut ::std::io::stderr(), $($arg)*);
        r.expect("failed printing to stderr");
    } }
);

pub fn parse(reader: &mut io::Read) -> (Vec<Request>,Vec<Response>) {
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

    (requests, responses)
}

pub fn parse_line(line: &String) -> Result<LogEvent, &'static str> {
    if line.contains("->") {
        let request = Request::new_from_log_line(line);

        return match request {
            Ok(request) => Ok(LogEvent::Request(request)),
            Err(err) => Err(err)
        }
    }

    if line.contains("<-") {
        let response = Response::new_from_log_line(line);

        return match response {
            Ok(response) => Ok(LogEvent::Response(response)),
            Err(err) => Err(err)
        }
    }

    Err("Line is neither a Request nor a Response")
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use super::*;
    use super::log_events::*;

    #[test]
    fn test_parse_simple() {
        let mut input_reader = File::open("src/test/simple-1.log").unwrap();

        let (requests, responses) = parse(&mut input_reader);

        assert_eq!(requests.len(), 2);
        assert_eq!(responses.len(), 2);
    }

    #[test]
    fn test_parse_ignore_broken_lines() {
        let mut input_reader = File::open("src/test/broken.log").unwrap();

        let (requests, responses) = parse(&mut input_reader);

        assert_eq!(requests.len(), 1);
        assert_eq!(responses.len(), 1);
    }

    #[test]
    fn test_parse_line_request() {
        let line = "08/Apr/2016:09:58:47 +0200 [02] -> GET /content/some/other.html HTTP/1.1".to_string();

        let event = match parse_line(&line).unwrap() {
            LogEvent::Request(request) => request,
            LogEvent::Response(_) => unreachable!(),
        };

        assert_eq!(event.id, 2);
    }

    #[test]
    fn test_parse_line_response() {
        let line = "08/Apr/2016:09:58:48 +0200 [05] <- 200 text/html 10ms".to_string();

        let event = match parse_line(&line).unwrap() {
            LogEvent::Request(_) => unreachable!(),
            LogEvent::Response(response) => response,
        };

        assert_eq!(event.id, 5);
    }
}
