use prometheus::{Registry, Gauge, Encoder, TextEncoder};

use super::*;
pub struct PrometheusRenderer<'a> {
    pub buffer: &'a mut Write,
    registry: prometheus::Registry,
    encoder: prometheus::TextEncoder,
    count: prometheus::Gauge,
}

impl<'a> PrometheusRenderer<'a> {
    pub fn new(stream: &'a mut Write) -> PrometheusRenderer<'a> {
        let registry = prometheus::Registry::new();

        let count = prometheus::Gauge::new("request_count", "The number of responses observed")
            .unwrap();

        registry.register(Box::new(count.clone()));

        PrometheusRenderer {
            registry: registry,
            buffer: stream,
            encoder: prometheus::TextEncoder::new(),
            count: count,
        }
    }
}

impl<'a> Renderer for PrometheusRenderer<'a> {
    fn render(&mut self, result: analyzer::RequestLogAnalyzerResult) {
        self.count.set(result.count as f64);

        let metric_familys = self.registry.gather();

        self.encoder.encode(&metric_familys, self.buffer);
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
        let mut mock_buffer = MockBuffer { write_calls: vec![] };

        let renderer = PrometheusRenderer::new(&mut mock_buffer);
    }

    #[test]
    fn test_render() {
        let mut mock_buffer = MockBuffer { write_calls: vec![] };

        let result = get_result_fixture();

        {
            let mut renderer = PrometheusRenderer::new(&mut mock_buffer);

            renderer.render(result);
        }

        println!("{:?}", mock_buffer.write_calls);

        assert_eq!(&mock_buffer.write_calls[0], "# HELP ");
        assert_eq!(&mock_buffer.write_calls[1], "request_count");
        assert_eq!(&mock_buffer.write_calls[2], " ");
        assert_eq!(&mock_buffer.write_calls[3],
                   "The number of responses observed");
        assert_eq!(&mock_buffer.write_calls[4], "\n");
        assert_eq!(&mock_buffer.write_calls[5], "# TYPE ");
        assert_eq!(&mock_buffer.write_calls[6], "request_count");
        assert_eq!(&mock_buffer.write_calls[7], " ");
        assert_eq!(&mock_buffer.write_calls[8], "gauge");
        assert_eq!(&mock_buffer.write_calls[9], "\n");
        assert_eq!(&mock_buffer.write_calls[10], "request_count");
        assert_eq!(&mock_buffer.write_calls[11], " ");
        assert_eq!(&mock_buffer.write_calls[12], "3");
        assert_eq!(&mock_buffer.write_calls[13], "\n");
    }
}
