use log::warn;
use ::prometheus::{Registry, Gauge, Encoder, TextEncoder};

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
    percentile99: prometheus::Gauge,
    client_error_4xx_rate: prometheus::Gauge,
    server_error_5xx_rate: prometheus::Gauge,
}

impl PrometheusRenderer {
    pub fn new() -> PrometheusRenderer {
        fn make_and_register_gauge(
            gauge_name: &str,
            registry: &prometheus::Registry,
        ) -> prometheus::Gauge {
            let gauge = prometheus::Gauge::new(
                String::from(gauge_name),
                format!("The {} of responses.", gauge_name),
            ).expect("Failed to create Prometheus gauge.");

            registry.register(Box::new(gauge.clone())).expect(
                "Failed to register Prometheus gauge.",
            );
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
            percentile99: make_and_register_gauge("request_time_percentile99", &registry),
            client_error_4xx_rate: make_and_register_gauge(
                "request_error_client_error_4xx_rate",
                &registry,
            ),
            server_error_5xx_rate: make_and_register_gauge(
                "request_error_server_error_5xx_rate",
                &registry,
            ),
            registry,
        }
    }
}

impl Renderer for PrometheusRenderer {
    fn render(&mut self, result: result::RequestLogAnalyzerResult) {
        self.count.set(result.count as f64);

        match result.timing {
            Some(timing) => {
                self.max.set(timing.max as f64);
                self.min.set(timing.min as f64);
                self.avg.set(timing.avg as f64);
                self.median.set(timing.median as f64);
                self.percentile90.set(timing.percentile90 as f64);
                self.percentile99.set(timing.percentile99 as f64);
            }
            None => {
                warn!("No matching log lines in file.");
            }
        }

        match result.error {
            Some(error) => {
                self.client_error_4xx_rate.set(
                    f64::from(error.client_error_4xx),
                );
                self.server_error_5xx_rate.set(
                    f64::from(error.server_error_5xx),
                );
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
    use crate::analyzer;

    #[test]
    fn test_render_1() {
        let result = result::RequestLogAnalyzerResult {
            count: 3,
            timing: Some(analyzer::TimingResult {
                max: 100,
                min: 1,
                avg: 37,
                median: 10,
                percentile90: 90,
                percentile99: 99,
                count: 3,
            }),
            error: Some(analyzer::aggregated_error_rates::ErrorRatesResult {
                client_error_4xx: 0.1,
                server_error_5xx: 0.2,
            }),
        };

        let mut renderer = PrometheusRenderer::new();
        renderer.render(result);

        let buffer_text = str::from_utf8(&renderer.buffer).unwrap();
        assert!(buffer_text.contains("request_count 3"));
        assert!(buffer_text.contains("request_time_max 100"));
        assert!(buffer_text.contains("request_time_min 1"));
        assert!(buffer_text.contains("request_time_avg 37"));
        assert!(buffer_text.contains("request_time_median 10"));
        assert!(buffer_text.contains("request_time_percentile90 90"));
        assert!(buffer_text.contains("request_time_percentile99 99"));
        assert!(buffer_text.contains(
            "request_error_client_error_4xx_rate 0.1",
        ));
        assert!(buffer_text.contains(
            "request_error_server_error_5xx_rate 0.2",
        ));
    }

    #[test]
    fn test_render_2() {
        let result = result::RequestLogAnalyzerResult {
            count: 300,
            timing: Some(analyzer::TimingResult {
                max: 1000,
                min: 10,
                avg: 42,
                median: 75,
                percentile90: 900,
                percentile99: 990,
                count: 300,
            }),
            error: None,
        };

        let mut renderer = PrometheusRenderer::new();
        renderer.render(result);

        let buffer_text = str::from_utf8(&renderer.buffer).unwrap();
        assert!(buffer_text.contains("request_count 300"));
        assert!(buffer_text.contains("request_time_max 1000"));
        assert!(buffer_text.contains("request_time_min 10"));
        assert!(buffer_text.contains("request_time_avg 42"));
        assert!(buffer_text.contains("request_time_median 75"));
        assert!(buffer_text.contains("request_time_percentile90 900"));
        assert!(buffer_text.contains("request_time_percentile99 990"));
    }

    #[test]
    fn test_render_no_lines() {
        let result = result::RequestLogAnalyzerResult {
            count: 0,
            timing: None,
            error: None,
        };

        let mut renderer = PrometheusRenderer::new();
        renderer.render(result);

        let buffer_text = str::from_utf8(&renderer.buffer).unwrap();

        assert!(buffer_text.contains("request_count 0"));
        assert!(buffer_text.contains("request_time_max 0"));
        assert!(buffer_text.contains("request_time_min 0"));
        assert!(buffer_text.contains("request_time_avg 0"));
        assert!(buffer_text.contains("request_time_median 0"));
        assert!(buffer_text.contains("request_time_percentile90 0"));
        assert!(buffer_text.contains("request_time_percentile99 0"));
    }
}
