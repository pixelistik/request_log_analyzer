use log_parser::log_events::HttpError;

#[derive(PartialEq, Debug)]
pub struct RequestLogAnalyzerResult {
    pub count: usize,
    pub max: usize,
    pub min: usize,
    pub avg: usize,
    pub median: usize,
    pub percentile90: usize,
}

#[derive(PartialEq, Debug)]
pub struct ErrorRatesResult {
    pub client_error_4xx: f32,
    pub server_error_5xx: f32,
}

pub trait HttpErrorState {
    fn error(&self) -> Option<HttpError>;
}

pub fn analyze<T>(statuses: &Vec<T>) -> Option<ErrorRatesResult>
    where T: HttpErrorState
{

    if statuses.is_empty() {
        return None;
    }

    let total_count = statuses.len() as f32;

    let client_error_4xx_count = statuses.iter()
        .map(|status| status.error())
        .filter(|status| *status == Some(HttpError::ClientError4xx))
        .count() as f32;

    let server_error_5xx_count = statuses.iter()
        .map(|status| status.error())
        .filter(|status| *status == Some(HttpError::ServerError5xx))
        .count() as f32;

    Some(ErrorRatesResult {
        client_error_4xx: client_error_4xx_count / total_count,
        server_error_5xx: server_error_5xx_count / total_count,
    })

}

#[cfg(test)]
mod tests {
    use super::*;

    impl HttpErrorState for String {
        fn error(&self) -> Option<HttpError> {
            match self.chars().nth(0) {
                Some('4') => Some(HttpError::ClientError4xx),
                Some('5') => Some(HttpError::ServerError5xx),
                _ => None,
            }
        }
    }

    #[test]
    fn test_analyze_all_ok() {
        let statuses: Vec<String> = vec![String::from("200")];

        let result = analyze(&statuses);

        let expected = Some(ErrorRatesResult {
            client_error_4xx: 0.0,
            server_error_5xx: 0.0,
        });

        assert_eq!(result, expected);
    }

    #[test]
    fn test_analyze_50_percent_client_errors() {
        let statuses: Vec<String> = vec![String::from("200"), String::from("403")];

        let result = analyze(&statuses);

        let expected = Some(ErrorRatesResult {
            client_error_4xx: 0.5,
            server_error_5xx: 0.0,
        });

        assert_eq!(result, expected);
    }

    #[test]
    fn test_analyze_50_percent_server_errors() {
        let statuses: Vec<String> = vec![String::from("200"), String::from("500")];

        let result = analyze(&statuses);

        let expected = Some(ErrorRatesResult {
            client_error_4xx: 0.0,
            server_error_5xx: 0.5,
        });

        assert_eq!(result, expected);
    }

    #[test]
    fn test_analyze_mixed_errors() {
        let statuses: Vec<String> = vec![String::from("404"),
                                         String::from("200"),
                                         String::from("500"),
                                         String::from("204")];

        let result = analyze(&statuses);

        let expected = Some(ErrorRatesResult {
            client_error_4xx: 0.25,
            server_error_5xx: 0.25,
        });

        assert_eq!(result, expected);
    }
}
