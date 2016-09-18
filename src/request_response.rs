use std::io;
use time::Tm;
use time::strptime;
use time::Duration;

use http_status::HttpStatus;

#[derive(Eq, PartialEq, Clone)]
#[derive(Debug)]
pub struct Request {
    pub id: i32,
    pub time: Tm,
    pub url: String,
}

impl Request {
    pub fn new_from_log_line(log_line: &String) -> Result<Request, io::Error> {
        let parts: Vec<&str> = log_line.split(" ").collect();


        let id = parts[2];
        let url = parts[5];

        Ok(Request {
            id: id[1..id.len()-1].parse().unwrap(),
            time: strptime(parts[0], "%d/%b/%Y:%H:%M:%S").unwrap(),
            url: url.to_string()
        })
    }

    pub fn get_matching_response<'a>(&'a self, responses: &'a Vec<Response>) -> Option<&Response> {
        match responses.binary_search_by_key(&self.id, |r| r.id) {
            Ok(index) => Some(&responses[index]),
            Err(_) => None
        }
    }
}

#[derive(Eq, PartialEq, Clone)]
#[derive(Debug)]
pub struct Response {
    pub id: i32,
    pub time: Tm,
    pub mime_type: String,
    pub response_time: Duration,
    pub http_status: HttpStatus,
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
            time: strptime(parts[0], "%d/%b/%Y:%H:%M:%S").unwrap(),
            response_time: Duration::milliseconds(response_time[0..response_time.len()-2].parse().unwrap()),
            mime_type: mime_type,
            http_status: HttpStatus::from_code(status_code.parse().unwrap()),
        })
    }
}

pub struct RequestResponsePair {
    pub request: Request,
    pub response: Response
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
    fn test_parse_response_line_inconsistent_space() {
        let line = "06/Apr/2016:14:54:16 +0200 [200] <- 200 text/html; charset=utf-8 250ms".to_string();

        let expected = Response {
            id: 200,
            time: strptime("06/Apr/2016:14:54:16 +0200", "%d/%b/%Y:%H:%M:%S").unwrap(),
            mime_type: "text/html; charset=utf-8".to_string(),
            response_time: Duration::milliseconds(250),
            http_status: HttpStatus::OK,
        };

        let result = Response::new_from_log_line(&line);

        assert_eq!(result.unwrap(), expected)
    }
}
