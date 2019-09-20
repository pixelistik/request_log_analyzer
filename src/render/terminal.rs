use std::io::prelude::*;
use crate::result;
use crate::render::Renderer;

pub struct TerminalRenderer<'a> {
    stream: &'a mut dyn Write,
}

impl<'a> TerminalRenderer<'a> {
    pub fn new(stream: &'a mut dyn Write) -> TerminalRenderer {
        TerminalRenderer { stream }
    }
}

impl<'a> Renderer for TerminalRenderer<'a> {
    fn render(&mut self, result: result::RequestLogAnalyzerResult) -> () {
        let mut write =
            |text: String| { let _ = self.stream.write(format!("{}\n", text).as_bytes()); };

        write(format!("count:\t{}", result.count));

        match result.timing {
            Some(timing) => {
                write(format!("time.avg:\t{}", timing.avg));
                write(format!("time.min:\t{}", timing.min));
                write(format!("time.median:\t{}", timing.median));
                write(format!("time.90percent:\t{}", timing.percentile90));
                write(format!("time.99percent:\t{}", timing.percentile99));
                write(format!("time.max:\t{}", timing.max));
            }
            None => warn!("No matching log lines for timing results."),
        }

        match result.error {
            Some(error) => {
                write(format!(
                    "error.client_error_4xx_rate:\t{}",
                    error.client_error_4xx
                ));
                write(format!(
                    "error.server_error_5xx_rate:\t{}",
                    error.server_error_5xx
                ));
            }
            None => warn!("No matching log lines for error rate results."),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::io::prelude::*;
    use std::str;
    use crate::analyzer;
    use super::*;

    struct MockWrite {
        write_calls: Vec<String>,
    }

    impl Write for MockWrite {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.write_calls.push(
                str::from_utf8(buf).unwrap().to_string(),
            );
            Ok(1)
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    fn get_result_fixture() -> result::RequestLogAnalyzerResult {
        result::RequestLogAnalyzerResult {
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
        }
    }

    #[test]
    fn test_terminal_renderer() {
        let mut mock_write = MockWrite { write_calls: vec![] };

        {
            let mut renderer = TerminalRenderer::new(&mut mock_write);
            let result = get_result_fixture();
            renderer.render(result);
        }
        println!("{:?}", mock_write.write_calls);
        assert!(mock_write.write_calls.contains(
            &String::from("time.max:\t100\n"),
        ));
        assert!(mock_write.write_calls.contains(
            &String::from("time.min:\t1\n"),
        ));
        assert!(mock_write.write_calls.contains(
            &String::from("time.avg:\t37\n"),
        ));
        assert!(mock_write.write_calls.contains(
            &String::from("time.median:\t10\n"),
        ));
        assert!(mock_write.write_calls.contains(&String::from(
            "time.90percent:\t90\n",
        )));
        assert!(mock_write.write_calls.contains(&String::from(
            "time.99percent:\t99\n",
        )));
        assert!(mock_write.write_calls.contains(
            &String::from("count:\t3\n"),
        ));

        assert!(mock_write.write_calls.contains(&String::from(
            "error.client_error_4xx_rate:\t0.1\n",
        )));
        assert!(mock_write.write_calls.contains(&String::from(
            "error.server_error_5xx_rate:\t0.2\n",
        )));
    }

    #[test]
    fn test_terminal_renderer_no_lines() {
        let mut mock_write = MockWrite { write_calls: vec![] };

        {
            let mut renderer = TerminalRenderer::new(&mut mock_write);

            let result = result::RequestLogAnalyzerResult {
                count: 0,
                timing: None,
                error: None,
            };

            renderer.render(result);
        }

        assert!(mock_write.write_calls.contains(
            &String::from("count:\t0\n"),
        ));
        assert_eq!(mock_write.write_calls.len(), 1);
    }
}
