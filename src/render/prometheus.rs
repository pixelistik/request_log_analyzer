use prometheus::{Registry, Gauge, Encoder, TextEncoder};

use super::*;
pub struct PrometheusRenderer {
    pub buffer: Vec<u8>,
    registry: prometheus::Registry,
    encoder: prometheus::TextEncoder,
    count: prometheus::Gauge,
}

impl PrometheusRenderer {
    pub fn new() -> PrometheusRenderer {
        let registry = prometheus::Registry::new();

        let count = prometheus::Gauge::new("request_count", "The number of responses observed")
            .unwrap();

        registry.register(Box::new(count.clone()));

        PrometheusRenderer {
            registry: registry,
            buffer: Vec::new(),
            encoder: prometheus::TextEncoder::new(),
            count: count,
        }
    }
}

impl Renderer for PrometheusRenderer {
    fn render(&mut self, result: analyzer::RequestLogAnalyzerResult) {
        self.count.set(result.count as f64);

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
        assert!(buffer_text.contains("request_count 3"))
    }
}
