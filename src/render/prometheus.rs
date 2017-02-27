use prometheus::{Registry, Gauge, Encoder, TextEncoder};
use std::collections::HashMap;

use super::*;
pub struct PrometheusRenderer {
    pub buffer: Vec<u8>,
    registry: prometheus::Registry,
    encoder: prometheus::TextEncoder,
    count: prometheus::Gauge,
    max: prometheus::Gauge,
    min: prometheus::Gauge,
    avg: prometheus::Gauge,
    median: prometheus::Gauge,
    percentile90: prometheus::Gauge,
}

impl PrometheusRenderer {
    pub fn new() -> PrometheusRenderer {
        let registry = prometheus::Registry::new();

        let gauge_names =
            vec!["count", "time_max", "time_min", "time_avg", "time_median", "time_percentile90"];
        let mut gauges = HashMap::new();

        for gauge_name in gauge_names {
            let gauge_name = format!("request_{}", gauge_name);
            let gauge = prometheus::Gauge::new(gauge_name.clone(),
                                               format!("The {} of response times.", gauge_name))
                .unwrap();
            registry.register(Box::new(gauge.clone()));
            gauges.insert(gauge_name, gauge);
        }

        PrometheusRenderer {
            registry: registry,
            buffer: Vec::new(),
            encoder: prometheus::TextEncoder::new(),
            count: gauges.remove("request_count").unwrap(),
            max: gauges.remove("request_time_max").unwrap(),
            min: gauges.remove("request_time_min").unwrap(),
            avg: gauges.remove("request_time_avg").unwrap(),
            median: gauges.remove("request_time_median").unwrap(),
            percentile90: gauges.remove("request_time_percentile90").unwrap(),
        }
    }
}

impl Renderer for PrometheusRenderer {
    fn render(&mut self, result: analyzer::RequestLogAnalyzerResult) {
        self.count.set(result.count as f64);
        self.max.set(result.max as f64);
        self.min.set(result.min as f64);
        self.avg.set(result.avg as f64);
        self.median.set(result.median as f64);
        self.percentile90.set(result.percentile90 as f64);

        let metric_familys = self.registry.gather();

        self.encoder.encode(&metric_familys, &mut self.buffer);
    }
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::io::prelude::*;
    use std::str;

    use super::*;

    struct MockBuffer {
        write_calls: Vec<String>,
    }

    impl Write for MockBuffer {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.write_calls.push(str::from_utf8(buf).unwrap().to_string());
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    fn get_result_fixture() -> analyzer::RequestLogAnalyzerResult {
        analyzer::RequestLogAnalyzerResult {
            count: 3,
            max: 100,
            min: 1,
            avg: 37,
            median: 10,
            percentile90: 100,
        }
    }

    #[test]
    fn test_instantiation() {
        let renderer = PrometheusRenderer::new();
    }

    #[test]
    fn test_render() {
        let result = get_result_fixture();

        let mut renderer = PrometheusRenderer::new();

        renderer.render(result);

        let buffer_text = str::from_utf8(&renderer.buffer).unwrap();
        assert!(buffer_text.contains("request_count 3"));
        assert!(buffer_text.contains("request_time_max 100"));
        assert!(buffer_text.contains("request_time_min 1"));
        assert!(buffer_text.contains("request_time_avg 37"));
        assert!(buffer_text.contains("request_time_median 10"));
        assert!(buffer_text.contains("request_time_percentile90 100"));
    }
}
