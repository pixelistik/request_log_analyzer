use crate::log_parser::log_events::HttpError;
use crate::request_response_matcher;
use crate::log_parser::*;

#[derive(PartialEq, Debug, Clone)]
pub struct ErrorRatesResult {
    pub client_error_4xx: f32,
    pub server_error_5xx: f32,
}

pub trait HttpErrorState {
    fn error(&self) -> Option<HttpError>;
}

impl HttpErrorState for request_response_matcher::RequestResponsePair {
    fn error(&self) -> Option<log_events::HttpError> {
        self.response.http_error.clone()
    }
}

impl HttpErrorState for Box<dyn HttpErrorState> {
    fn error(&self) -> Option<log_events::HttpError> {
        (**self).error()
    }
}

pub struct AggregatedErrorRates {
    total_count: usize,
    client_error_4xx_count: usize,
    server_error_5xx_count: usize,
}

impl AggregatedErrorRates {
    pub fn new() -> AggregatedErrorRates {
        AggregatedErrorRates {
            total_count: 0,
            client_error_4xx_count: 0,
            server_error_5xx_count: 0,
        }
    }

    pub fn add<T>(&mut self, value: &T)
    where
        T: HttpErrorState,
    {
        self.total_count += 1;

        match value.error() {
            Some(HttpError::ClientError4xx) => self.client_error_4xx_count += 1,
            Some(HttpError::ServerError5xx) => self.server_error_5xx_count += 1,
            None => (),
        }
    }

    pub fn result(&self) -> Option<ErrorRatesResult> {
        if self.total_count == 0 {
            return None;
        }

        Some(ErrorRatesResult {
            client_error_4xx: (self.client_error_4xx_count as f32 / self.total_count as f32 *
                                   10000.0)
                .round() / 10000.0,
            server_error_5xx: (self.server_error_5xx_count as f32 / self.total_count as f32 *
                                   10000.0)
                .round() / 10000.0,
        })
    }
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
    fn test_all_ok() {
        let mut error_rates = AggregatedErrorRates::new();

        error_rates.add(&String::from("200"));
        error_rates.add(&String::from("200"));

        let result = error_rates.result();
        let expected = Some(ErrorRatesResult {
            client_error_4xx: 0.0,
            server_error_5xx: 0.0,
        });
        assert_eq!(result, expected);
    }

    #[test]
    fn test_50_percent_client_errors() {
        let mut error_rates = AggregatedErrorRates::new();

        error_rates.add(&String::from("200"));
        error_rates.add(&String::from("403"));

        let result = error_rates.result();
        let expected = Some(ErrorRatesResult {
            client_error_4xx: 0.5,
            server_error_5xx: 0.0,
        });
        assert_eq!(result, expected);
    }

    #[test]
    fn test_50_percent_server_errors() {
        let mut error_rates = AggregatedErrorRates::new();

        error_rates.add(&String::from("201"));
        error_rates.add(&String::from("500"));

        let result = error_rates.result();
        let expected = Some(ErrorRatesResult {
            client_error_4xx: 0.0,
            server_error_5xx: 0.5,
        });
        assert_eq!(result, expected);
    }

    #[test]
    fn test_mixed_errors() {
        let mut error_rates = AggregatedErrorRates::new();

        error_rates.add(&String::from("302"));
        error_rates.add(&String::from("500"));
        error_rates.add(&String::from("403"));
        error_rates.add(&String::from("200"));

        let result = error_rates.result();
        let expected = Some(ErrorRatesResult {
            client_error_4xx: 0.25,
            server_error_5xx: 0.25,
        });
        assert_eq!(result, expected);
    }

    #[test]
    fn test_empty() {
        let error_rates = AggregatedErrorRates::new();

        let result = error_rates.result();
        assert_eq!(result, None);
    }

    #[test]
    fn test_rounding() {
        let mut error_rates = AggregatedErrorRates::new();

        error_rates.add(&String::from("200"));
        error_rates.add(&String::from("500"));
        error_rates.add(&String::from("403"));

        let result = error_rates.result();
        let expected = Some(ErrorRatesResult {
            client_error_4xx: 0.3333,
            server_error_5xx: 0.3333,
        });
        assert_eq!(result, expected);
    }
}
