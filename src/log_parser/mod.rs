use std::io;
use std::io::BufRead;
use std::io::Write;

mod log_events;
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

#[cfg(test)]
mod tests {
    use std::fs::File;
    use super::*;

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
}
