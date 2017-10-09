use aggregated_stats;
use result;

pub mod aggregated_error_rates;

#[derive(PartialEq, Debug)]
pub struct TimingResult {
    pub max: usize,
    pub min: usize,
    pub avg: usize,
    pub median: usize,
    pub percentile90: usize,
    pub count: usize,
}

pub trait Timing {
    fn num_milliseconds(&self) -> i64;
}

pub fn analyze_iterator<I, T>(timings: I) -> result::RequestLogAnalyzerResult
    where I: Iterator<Item = T>,
          T: Timing + aggregated_error_rates::HttpErrorState
{
    let mut stats = aggregated_stats::AggregatedStats::new();
    let mut error_rates = aggregated_error_rates::AggregatedErrorRates::new();

    for timing in timings {
        stats.add(timing.num_milliseconds() as usize);
        error_rates.add(&timing);
    }

    if stats.max().is_none() {
        return result::RequestLogAnalyzerResult {
            count: 0,
            timing: None,
            error: None,
        };
    }

    result::RequestLogAnalyzerResult {
        count: stats.count(),
        timing: Some(TimingResult {
            max: stats.max().unwrap(),
            min: stats.min().unwrap(),
            avg: stats.average().unwrap() as usize,
            median: stats.median().unwrap() as usize,
            percentile90: stats.quantile(0.9).unwrap() as usize,
            count: stats.count(),
        }),
        error: error_rates.result(),
    }
}

#[cfg(test)]
mod tests {
    use result;
    use timing_analyzer::aggregated_error_rates::HttpErrorState;
    use timing_analyzer::aggregated_error_rates::ErrorRatesResult;
    use log_parser::log_events::HttpError;
    use super::*;

    impl Timing for i64 {
        fn num_milliseconds(&self) -> i64 {
            self.clone()
        }
    }

    impl HttpErrorState for i64 {
        fn error(&self) -> Option<HttpError> {
            None
        }
    }

    #[test]
    fn test_analyze_iterator() {
        let times: Vec<i64> = vec![1, 10, 100];
        let times_iterator = times.into_iter();

        let result = analyze_iterator(times_iterator);

        let expected = result::RequestLogAnalyzerResult {
            count: 3,
            timing: Some(TimingResult {
                max: 100,
                min: 1,
                avg: 37,
                median: 10,
                percentile90: 100,
                count: 3,
            }),
            error: Some(ErrorRatesResult {
                client_error_4xx: 0.0,
                server_error_5xx: 0.0,
            }),
        };

        assert_eq!(result, expected);
    }

    #[test]
    fn test_analyze_empty_iterator() {
        let times: Vec<i64> = vec![];
        let times_iterator = times.into_iter();

        let result = analyze_iterator(times_iterator);

        let expected = result::RequestLogAnalyzerResult {
            count: 0,
            timing: None,
            error: None,
        };

        assert_eq!(result, expected);
    }
}
