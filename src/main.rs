extern crate regex;
use regex::Regex;

use std::io::{self, BufReader};
use std::io::BufRead;
use std::io::Read;
use std::fs::File;
extern crate time;
use time::Tm;
use time::strptime;

#[derive(Eq, PartialEq)]
#[derive(Debug)]
pub struct Request {
    time: Tm,
    url: String
}

fn open_logfile(path: &str) -> Result<String, io::Error> {
    let mut f = try!(File::open("src/test/simple-1.log"));

    let f = BufReader::new(f);

    let mut requests: Vec<Request> = Vec::new();

    for line in f.lines() {
        let r = try!(parse_line(line.unwrap()));

        requests.push(r)
    }

    println!("So many: {}", requests.len());

    Ok("bla".to_string())
}

pub fn parse_line(log_line: String) -> Result<Request, io::Error> {
    let parts: Vec<&str> = log_line.split(" ").collect();

    let url = parts[5];

    Ok(Request {
        time: strptime(parts[0], "%d/%b/%Y:%H:%M:%S").unwrap(),
        url: url.to_string()
    })

}

fn main() {
    println!("{}", open_logfile("some").unwrap());
}

#[cfg(test)]
mod tests {
	use super::*;
    extern crate time;
    use time::strptime;

    #[test]
    fn test_parse_line() {
        let line = "08/Apr/2016:09:58:47 +0200 [02] -> GET /content/some/other.html HTTP/1.1".to_string();

        let expected = Request {
            time: strptime("08/Apr/2016:09:58:47 +0200", "%d/%b/%Y:%H:%M:%S").unwrap(),
            url: "/content/some/other.html".to_string()
        };

        let result = parse_line(line);

        assert_eq!(result.unwrap(), expected)
    }
}
