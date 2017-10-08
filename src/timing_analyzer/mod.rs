use aggregated_stats;
use result;

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
          T: Timing
{
    let mut stats = aggregated_stats::AggregatedStats::new();

    for timing in timings {
        stats.add(timing.num_milliseconds() as usize);
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
        error: None,
    }
}

#[cfg(test)]
mod tests {
    use result;
    use super::*;

    impl Timing for i64 {
        fn num_milliseconds(&self) -> i64 {
            self.clone()
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
            error: None,
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
