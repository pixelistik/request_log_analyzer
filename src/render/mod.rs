use std::io;
use std::io::prelude::*;
use chrono::*;
use analyzer;

pub fn render_terminal(result: analyzer::RequestLogAnalyzerResult) {
    println!("count:\t{}", result.count);
    println!("time.avg:\t{}", result.avg);
    println!("time.min:\t{}", result.min);
    println!("time.median:\t{}", result.median);
    println!("time.90percent:\t{}", result.percentile90);
    println!("time.max:\t{}", result.max);
}

pub fn render_graphite<T: Write>(result: analyzer::RequestLogAnalyzerResult, time: DateTime<FixedOffset>, prefix: Option<&str>, mut stream: T) {
    let prefix_text: &str;
    let prefix_separator: &str;

    match prefix {
        Some(p) => {
            prefix_text = p;
            prefix_separator = ".";
        }
        None => {
            prefix_text = "";
            prefix_separator = "";
        }
    };

    let mut write = |text: String| {
        let _ = stream.write(
            format!("{}{}{} {}\n", prefix_text, prefix_separator, text, time.timestamp() )
            .as_bytes()
        );
    };

    write(format!("requests.count {}", result.count));
    write(format!("requests.time.max {}", result.max));
    write(format!("requests.time.min {}", result.min));
    write(format!("requests.time.avg {}", result.avg));
    write(format!("requests.time.median {}", result.median));
    write(format!("requests.time.90percent {}", result.percentile90));
}

mod tests {
    use std::io;
    use std::io::prelude::*;
    use std::str;
    use chrono::*;
    use analyzer;
    use super::*;

    struct MockTcpStream {
        write_calls: Vec<String>,
    }

    impl Write for MockTcpStream {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.write_calls.push(str::from_utf8(buf).unwrap().to_string());
            Ok(1)
        }

        fn flush(&mut self) -> io::Result<()> { Ok(()) }
    }

    #[test]
    fn test_render_graphite() {
        let mut mock_tcp_stream = MockTcpStream {
            write_calls: vec![]
        };

        render_graphite(analyzer::RequestLogAnalyzerResult {
                count: 3,
                max: 100,
                min: 1,
                avg: 37,
                median: 10,
                percentile90: 100,
            },
            DateTime::parse_from_str("22/Sep/2016:22:41:59 +0200", "%d/%b/%Y:%H:%M:%S %z").unwrap(),
            None,
            &mut mock_tcp_stream
        );

        assert_eq!(&mock_tcp_stream.write_calls[0], "requests.count 3 1474576919\n");
        assert_eq!(&mock_tcp_stream.write_calls[1], "requests.time.max 100 1474576919\n");
        assert_eq!(&mock_tcp_stream.write_calls[2], "requests.time.min 1 1474576919\n");
        assert_eq!(&mock_tcp_stream.write_calls[3], "requests.time.avg 37 1474576919\n");
        assert_eq!(&mock_tcp_stream.write_calls[4], "requests.time.median 10 1474576919\n");
        assert_eq!(&mock_tcp_stream.write_calls[5], "requests.time.90percent 100 1474576919\n");
    }
}
