use std::io::{self, BufReader};
use std::io::BufRead;
use std::fs::File;
extern crate time;
use time::Tm;
use time::strptime;
use time::Duration;

mod http_status;
use http_status::HttpStatus;

#[derive(Eq, PartialEq, Clone)]
#[derive(Debug)]
pub struct Request {
    id: i32,
    time: Tm,
    url: String,
}

impl Request {
    fn new_from_log_line(log_line: &String) -> Result<Request, io::Error> {
        let parts: Vec<&str> = log_line.split(" ").collect();


        let id = parts[2];
        let url = parts[5];

        Ok(Request {
            id: id[1..id.len()-1].parse().unwrap(),
            time: strptime(parts[0], "%d/%b/%Y:%H:%M:%S").unwrap(),
            url: url.to_string()
        })
    }

    fn get_matching_response<'a>(&'a self, responses: &'a Vec<Response>) -> Option<&Response> {
        for response in responses {
            if self.id == response.id {
                return Some(response)
            }
        }

        None
    }
}

#[derive(Eq, PartialEq, Clone)]
#[derive(Debug)]
pub struct Response {
    id: i32,
    time: Tm,
    mime_type: String,
    response_time: Duration,
    http_status: HttpStatus,
}

impl Response {
    fn new_from_log_line(log_line: &String) -> Result<Response, io::Error> {
        let parts: Vec<&str> = log_line.split(" ").collect();

        let id = parts[2];
        let response_time = parts[parts.len()-1];
        let mime_type = parts[5];
        let status_code = parts[4];

        Ok(Response {
            id: id[1..id.len()-1].parse().unwrap(),
            time: strptime(parts[0], "%d/%b/%Y:%H:%M:%S").unwrap(),
            response_time: Duration::milliseconds(response_time[0..response_time.len()-2].parse().unwrap()),
            mime_type: mime_type.to_string(),
            http_status: HttpStatus::from_code(status_code.parse().unwrap()),
        })
    }
}

pub fn open_logfile(path: &str) -> Result<(Vec<Request>,Vec<Response>), io::Error> {
    let f = try!(File::open(path));

    let f = BufReader::new(f);

    let mut requests: Vec<Request> = Vec::new();
    let mut responses: Vec<Response> = Vec::new();


    for line in f.lines() {
        let line_value = &line.unwrap();

        if line_value.contains("->") {
            let r = try!(Request::new_from_log_line(&line_value));
            println!("{:#?}", r);
            requests.push(r)
        }

        if line_value.contains("<-") {
            let r = try!(Response::new_from_log_line(&line_value));
            println!("{:#?}", r);
            responses.push(r)
        }

    }

    Ok((requests, responses))
}

pub struct RequestResponsePair {
    request: Request,
    response: Response
}

pub fn pair_requests_responses(requests:Vec<Request>, responses: Vec<Response>) -> Vec<RequestResponsePair> {
    let mut request_response_pairs: Vec<RequestResponsePair> = Vec::new();

    for request in requests  {
        match request.get_matching_response(&responses) {
            Some(response) => request_response_pairs.push(RequestResponsePair{
                request: request.clone(),
                response: response.clone()
            }),
            None => println!("none"),
        }
    }

    request_response_pairs
}


fn main() {
    let lines = open_logfile("src/test/simple-1.log");
    let (requests, responses) = lines.unwrap();
    println!("So many requests: {}", requests.len());
    println!("So many responses: {}", responses.len());

    let pairs: Vec<RequestResponsePair> = pair_requests_responses(requests, responses);

    let times: Vec<i64> = pairs.iter().map(|rr| rr.response.response_time.num_milliseconds()).collect();
    let sum: i64 = times.iter().sum();
    println!("{}", sum);
}

#[cfg(test)]
mod tests {
	use super::*;
    extern crate time;
    use time::strptime;
    use::time::Duration;
    use http_status::HttpStatus;

    #[test]
    fn test_parse_request_line() {
        let line = "08/Apr/2016:09:58:47 +0200 [02] -> GET /content/some/other.html HTTP/1.1".to_string();

        let expected = Request {
            id: 2,
            time: strptime("08/Apr/2016:09:58:47 +0200", "%d/%b/%Y:%H:%M:%S").unwrap(),
            url: "/content/some/other.html".to_string()
        };

        let result = Request::new_from_log_line(&line);

        assert_eq!(result.unwrap(), expected)
    }

    #[test]
    fn test_parse_response_line() {
        let line = "08/Apr/2016:09:58:48 +0200 [02] <- 200 text/html 10ms".to_string();

        let expected = Response {
            id: 2,
            time: strptime("08/Apr/2016:09:58:48 +0200", "%d/%b/%Y:%H:%M:%S").unwrap(),
            mime_type: "text/html".to_string(),
            response_time: Duration::milliseconds(10),
            http_status: HttpStatus::OK,
        };

        let result = Response::new_from_log_line(&line);

        assert_eq!(result.unwrap(), expected)
    }

    #[test]
    fn test_open_logfile() {
        let lines = open_logfile("src/test/simple-1.log");
        let (requests, responses) = lines.unwrap();

        assert_eq!(requests.len(), 2);
        assert_eq!(responses.len(), 2);
    }

    #[test]
    fn test_get_matching_response() {
        let lines = open_logfile("src/test/simple-1.log");
        let (requests, responses) = lines.unwrap();

        let result = requests[0].get_matching_response(&responses);

        let expected = Response {
            id: 1,
            time: strptime("08/Apr/2016:09:57:47 +0200", "%d/%b/%Y:%H:%M:%S").unwrap(),
            mime_type: "text/html".to_string(),
            response_time: Duration::milliseconds(7),
            http_status: HttpStatus::OK,
        };

        assert_eq!(*result.unwrap(), expected);
    }

    #[test]
    fn test_get_matching_response_none_found() {
        let lines = open_logfile("src/test/simple-1.log");
        let (_, responses) = lines.unwrap();

        let request_without_matching = Request {
            id: 999,
            time: strptime("08/Apr/2016:09:58:47 +0200", "%d/%b/%Y:%H:%M:%S").unwrap(),
            url: "/content/some/other.html".to_string()
        };

        let result = request_without_matching.get_matching_response(&responses);

        assert!(result.is_none());
    }

    #[test]
    fn test_pair_requests_resonses() {
        let lines = open_logfile("src/test/simple-1.log");
        let (requests, responses) = lines.unwrap();

        let result = pair_requests_responses(requests, responses);

        assert_eq!(result.len(), 2);

        assert_eq!(result[0].request.id, result[0].response.id);
        assert_eq!(result[1].request.id, result[1].response.id);
    }
}
