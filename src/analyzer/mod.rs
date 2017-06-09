use stats::median;

pub mod percentile;

#[derive(PartialEq, Debug)]
pub struct RequestLogAnalyzerResult {
    pub count: usize,
    pub max: usize,
    pub min: usize,
    pub avg: usize,
    pub median: usize,
    pub percentile90: usize,
}

pub trait Timing {
    fn num_milliseconds(&self) -> i64;
}

pub fn analyze<T>(timings: &Vec<T>) -> Option<RequestLogAnalyzerResult>
    where T: Timing
{

    if timings.is_empty() {
        return None;
    }

    let times: Vec<i64> = timings.iter()
        .map(|timing| timing.num_milliseconds())
        .collect();


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
    use super::*;

    impl Timing for i64 {
        fn num_milliseconds(&self) -> i64 {
            self.clone()
        }
    }

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
        let times: Vec<i64> = vec![];

        let result = analyze(&times);

        let expected = None;

        assert_eq!(result, expected);
    }
}
