use prometheus::{Registry, Gauge, Encoder, TextEncoder};

use super::*;
pub struct PrometheusRenderer {
    pub buffer: Vec<u8>,
    registry: prometheus::Registry,
    pub encoder: prometheus::TextEncoder,
    count: prometheus::Gauge,
    max: prometheus::Gauge,
    min: prometheus::Gauge,
    avg: prometheus::Gauge,
    median: prometheus::Gauge,
    percentile90: prometheus::Gauge,
}

impl PrometheusRenderer {
    pub fn new() -> PrometheusRenderer {
        fn make_and_register_gauge(gauge_name: &str,
                                   registry: &prometheus::Registry)
                                   -> prometheus::Gauge {
            let gauge = prometheus::Gauge::new(String::from(gauge_name),
                                               format!("The {} of response times.", gauge_name))
                .expect("Failed to create Prometheus gauge.");

            registry.register(Box::new(gauge.clone()))
                .expect("Failed to register Prometheus gauge.");
            gauge
        }

        let registry = prometheus::Registry::new();

        PrometheusRenderer {
            buffer: Vec::new(),
            encoder: prometheus::TextEncoder::new(),
            count: make_and_register_gauge("request_count", &registry),
            max: make_and_register_gauge("request_time_max", &registry),
            min: make_and_register_gauge("request_time_min", &registry),
            avg: make_and_register_gauge("request_time_avg", &registry),
            median: make_and_register_gauge("request_time_median", &registry),
            percentile90: make_and_register_gauge("request_time_percentile90", &registry),
            registry: registry,
        }
    }
}

impl Renderer for PrometheusRenderer {
    fn render(&mut self, result: Option<analyzer::RequestLogAnalyzerResult>) {
        match result {
            Some(result) => {
                self.count.set(result.count as f64);
                self.max.set(result.max as f64);
                self.min.set(result.min as f64);
                self.avg.set(result.avg as f64);
                self.median.set(result.median as f64);
                self.percentile90.set(result.percentile90 as f64);
            }
            None => {
                warn!("No matching log lines in file.");
            }
        }
        let metric_familys = self.registry.gather();

        self.encoder
            .encode(&metric_familys, &mut self.buffer)
            .expect("Failed to encode Prometheus metrics.");
    }
}

#[cfg(test)]
mod tests {
    use std::str;
    use super::*;

    #[test]
    fn test_render_1() {
        let result = Some(analyzer::RequestLogAnalyzerResult {
            count: 3,
            max: 100,
            min: 1,
            avg: 37,
            median: 10,
            percentile90: 100,
        });

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

    #[test]
    fn test_render_2() {
        let result = Some(analyzer::RequestLogAnalyzerResult {
            count: 300,
            max: 1000,
            min: 10,
            avg: 42,
            median: 75,
            percentile90: 900,
        });

        let mut renderer = PrometheusRenderer::new();
        renderer.render(result);

        let buffer_text = str::from_utf8(&renderer.buffer).unwrap();
        assert!(buffer_text.contains("request_count 300"));
        assert!(buffer_text.contains("request_time_max 1000"));
        assert!(buffer_text.contains("request_time_min 10"));
        assert!(buffer_text.contains("request_time_avg 42"));
        assert!(buffer_text.contains("request_time_median 75"));
        assert!(buffer_text.contains("request_time_percentile90 900"));
    }

    #[test]
    fn test_render_empty() {
        let result = None;

        let mut renderer = PrometheusRenderer::new();
        renderer.render(result);

        let buffer_text = str::from_utf8(&renderer.buffer).unwrap();
        assert!(buffer_text.contains("request_count 0"));
        // @TODO: Does NaN really exist with Prometheus?
        // https://prometheus.io/docs/practices/instrumentation/#avoid-missing-metrics
        // assert!(buffer_text.contains("request_time_max NaN"));
        // assert!(buffer_text.contains("request_time_min NaN"));
        // assert!(buffer_text.contains("request_time_avg NaN"));
        // assert!(buffer_text.contains("request_time_median NaN"));
        // assert!(buffer_text.contains("request_time_percentile90 NaN"));
    }
}
