use stats::median;
use request_response_matcher::*;

pub mod percentile;

#[derive(PartialEq)]
#[derive(Debug)]
pub struct RequestLogAnalyzerResult {
    pub count: usize,
    pub max: usize,
    pub min: usize,
    pub avg: usize,
    pub median: usize,
    pub percentile90: usize,
}

pub fn analyze(times: &Vec<i64>) -> Option<RequestLogAnalyzerResult> {
    if times.len() == 0 {
        return None;
    }

    let sum: usize = times.iter().sum::<i64>() as usize;
    let avg: usize = sum / times.len();

    let max: usize = *times.iter().max().unwrap() as usize;
    let min: usize = *times.iter().min().unwrap() as usize;

    let percentile90: usize = percentile::percentile(&times, 0.9) as usize;

    let median = median(times.iter().cloned()).unwrap() as usize;

    Some(RequestLogAnalyzerResult {
        count: times.len().into(),
        max: max,
        min: min,
        avg: avg,
        median: median,
        percentile90: percentile90,
    })
}

#[cfg(test)]
mod tests {
    use log_parser::log_events::*;
    use request_response_matcher::*;
    use super::*;

    #[test]
    fn test_analyze() {
        let times: Vec<i64> = vec![1, 10, 100];

        let result = analyze(&times);

        let expected = Some(RequestLogAnalyzerResult {
            count: 3,
            max: 100,
            min: 1,
            avg: 37,
            median: 10,
            percentile90: 100,
        });

        assert_eq!(result, expected);
    }

    #[test]
    fn test_analyze_empty() {
        let times = vec![];

        let result = analyze(&times);

        let expected = None;

        assert_eq!(result, expected);
    }
}
