use std::io::{self, BufReader};
use std::io::BufRead;
use std::fs::File;
extern crate time;
use time::Tm;
use time::strptime;
use time::Duration;

#[derive(Eq, PartialEq, Clone)]
#[derive(Debug)]
pub enum HttpStatus {
    Continue,
    SwitchingProtocols,
    Processing,
    OK,
    Created,
    Accepted,
    NonAuthoritativeInformation,
    NoContent,
    ResetContent,
    PartialContent,
    MultiStatus,
    AlreadyReported,
    IMUsed,
    MultipleChoices,
    MovedPermanently,
    Found,
    SeeOther,
    NotModified,
    UseProxy,
    TemporaryRedirect,
    PermanentRedirect,
    BadRequest,
    Unauthorized,
    PaymentRequired,
    Forbidden,
    NotFound,
    MethodNotAllowed,
    NotAcceptable,
    ProxyAuthenticationRequired,
    RequestTimeout,
    Conflict,
    Gone,
    LengthRequired,
    PreconditionFailed,
    PayloadTooLarge,
    URITooLong,
    UnsupportedMediaType,
    RangeNotSatisfiable,
    ExpectationFailed,
    ImaTeapot,
    MisdirectedRequest,
    UnprocessableEntity,
    Locked,
    FailedDependency,
    UpgradeRequired,
    PreconditionRequired,
    TooManyRequests,
    RequestHeaderFieldsTooLarge,
    UnavailableForLegalReasons,
    InternalServerError,
    NotImplemented,
    BadGateway,
    ServiceUnavailable,
    GatewayTimeout,
    HTTPVersionNotSupported,
    VariantAlsoNegotiates,
    InsufficientStorage,
    LoopDetected,
    NotExtended,
    NetworkAuthenticationRequired,
    Unregistered,
}

impl HttpStatus {
    fn from_code(code: i32) -> HttpStatus {
        match code {
            100 => HttpStatus::Continue,
            101 => HttpStatus::SwitchingProtocols,
            102 => HttpStatus::Processing,
            200 => HttpStatus::OK,
            201 => HttpStatus::Created,
            202 => HttpStatus::Accepted,
            203 => HttpStatus::NonAuthoritativeInformation,
            204 => HttpStatus::NoContent,
            205 => HttpStatus::ResetContent,
            206 => HttpStatus::PartialContent,
            207 => HttpStatus::MultiStatus,
            208 => HttpStatus::AlreadyReported,
            226 => HttpStatus::IMUsed,
            300 => HttpStatus::MultipleChoices,
            301 => HttpStatus::MovedPermanently,
            302 => HttpStatus::Found,
            303 => HttpStatus::SeeOther,
            304 => HttpStatus::NotModified,
            305 => HttpStatus::UseProxy,
            307 => HttpStatus::TemporaryRedirect,
            308 => HttpStatus::PermanentRedirect,
            400 => HttpStatus::BadRequest,
            401 => HttpStatus::Unauthorized,
            402 => HttpStatus::PaymentRequired,
            403 => HttpStatus::Forbidden,
            404 => HttpStatus::NotFound,
            405 => HttpStatus::MethodNotAllowed,
            406 => HttpStatus::NotAcceptable,
            407 => HttpStatus::ProxyAuthenticationRequired,
            408 => HttpStatus::RequestTimeout,
            409 => HttpStatus::Conflict,
            410 => HttpStatus::Gone,
            411 => HttpStatus::LengthRequired,
            412 => HttpStatus::PreconditionFailed,
            413 => HttpStatus::PayloadTooLarge,
            414 => HttpStatus::URITooLong,
            415 => HttpStatus::UnsupportedMediaType,
            416 => HttpStatus::RangeNotSatisfiable,
            417 => HttpStatus::ExpectationFailed,
            418 => HttpStatus::ImaTeapot,
            421 => HttpStatus::MisdirectedRequest,
            422 => HttpStatus::UnprocessableEntity,
            423 => HttpStatus::Locked,
            424 => HttpStatus::FailedDependency,
            426 => HttpStatus::UpgradeRequired,
            428 => HttpStatus::PreconditionRequired,
            429 => HttpStatus::TooManyRequests,
            431 => HttpStatus::RequestHeaderFieldsTooLarge,
            451 => HttpStatus::UnavailableForLegalReasons,
            500 => HttpStatus::InternalServerError,
            501 => HttpStatus::NotImplemented,
            502 => HttpStatus::BadGateway,
            503 => HttpStatus::ServiceUnavailable,
            504 => HttpStatus::GatewayTimeout,
            505 => HttpStatus::HTTPVersionNotSupported,
            506 => HttpStatus::VariantAlsoNegotiates,
            507 => HttpStatus::InsufficientStorage,
            508 => HttpStatus::LoopDetected,
            510 => HttpStatus::NotExtended,
            511 => HttpStatus::NetworkAuthenticationRequired,
            _ => HttpStatus::Unregistered,
        }
    }
}

#[derive(Eq, PartialEq, Clone)]
#[derive(Debug)]
pub struct Request {
    id: i32,
    time: Tm,
    url: String,
}

impl Request {
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

pub fn open_logfile(path: &str) -> Result<(Vec<Request>,Vec<Response>), io::Error> {
    let f = try!(File::open(path));

    let f = BufReader::new(f);

    let mut requests: Vec<Request> = Vec::new();
    let mut responses: Vec<Response> = Vec::new();


    for line in f.lines() {
        let line_value = &line.unwrap();

        if line_value.contains("->") {
            let r = try!(parse_request_line(&line_value));
            println!("{:#?}", r);
            requests.push(r)
        }

        if line_value.contains("<-") {
            let r = try!(parse_response_line(&line_value));
            println!("{:#?}", r);
            responses.push(r)
        }

    }

    Ok((requests, responses))
}

pub fn parse_request_line(log_line: &String) -> Result<Request, io::Error> {
    let parts: Vec<&str> = log_line.split(" ").collect();


    let id = parts[2];
    let url = parts[5];

    Ok(Request {
        id: id[1..id.len()-1].parse().unwrap(),
        time: strptime(parts[0], "%d/%b/%Y:%H:%M:%S").unwrap(),
        url: url.to_string()
    })
}

pub fn parse_response_line(log_line: &String) -> Result<Response, io::Error> {
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

pub struct RequestResponsePair {
    request: Request,
    response: Response
}

pub fn pair_requests_responses(mut requests:Vec<Request>, responses: Vec<Response>) -> Vec<RequestResponsePair> {
    let mut request_response_pairs: Vec<RequestResponsePair> = Vec::new();

    for request in requests.drain(..) {
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
}

#[cfg(test)]
mod tests {
	use super::*;
    extern crate time;
    use time::strptime;
    use::time::Duration;

    #[test]
    fn test_parse_request_line() {
        let line = "08/Apr/2016:09:58:47 +0200 [02] -> GET /content/some/other.html HTTP/1.1".to_string();

        let expected = Request {
            id: 2,
            time: strptime("08/Apr/2016:09:58:47 +0200", "%d/%b/%Y:%H:%M:%S").unwrap(),
            url: "/content/some/other.html".to_string()
        };

        let result = parse_request_line(&line);

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

        let result = parse_response_line(&line);

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
        let (requests, responses) = lines.unwrap();

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
    }
}
