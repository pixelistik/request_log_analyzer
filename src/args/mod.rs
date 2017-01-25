use clap::{Arg, App};
use chrono::*;
use filter;

#[derive(Debug)]
#[derive(PartialEq)]
pub struct RequestLogAnalyzerArgs {
    pub filename: String,
    pub conditions: filter::FilterConditions,
    pub graphite_server: Option<String>,
    pub graphite_port: Option<u16>,
    pub graphite_prefix: Option<String>,
}

pub fn parse_args<'a, T>(args: T) -> Result<RequestLogAnalyzerArgs, &'a str>
    where T: IntoIterator<Item = String>
{
    let app = App::new("Request.log Analyzer")
        .author(crate_authors!())
        .version(crate_version!())
        .arg(Arg::with_name("filename")
            .index(1)
            .value_name("FILE")
            .required(false)
            .help("Log file to analyze, defaults to stdin")
            .takes_value(true))
        .arg(Arg::with_name("time_filter_minutes")
            .value_name("MINUTES")
            .short("t")
            .help("Limit to the last n minutes")
            .takes_value(true))
        .arg(Arg::with_name("include_term")
            .value_name("TERM")
            .long("include")
            .help("Only include lines that contain one of these terms")
            .takes_value(true)
            .multiple(true)
            .number_of_values(1))
        .arg(Arg::with_name("exclude_term")
            .value_name("TERM")
            .long("exclude")
            .help("Exclude lines that contain one of these terms")
            .takes_value(true)
            .multiple(true)
            .number_of_values(1))
        .arg(Arg::with_name("graphite-server")
            .value_name("GRAPHITE_SERVER")
            .long("graphite-server")
            .help("Send values to this Graphite server instead of stdout")
            .takes_value(true))
        .arg(Arg::with_name("graphite-port")
            .value_name("GRAPHITE_PORT")
            .long("graphite-port")
            .takes_value(true)
            .default_value("2003"))
        .arg(Arg::with_name("graphite-prefix")
            .value_name("GRAPHITE_PREFIX")
            .long("graphite-prefix")
            .help("Prefix for Graphite key, e.g. 'servers.prod.publisher1'")
            .takes_value(true))
        .get_matches_from(args);

    let filename = app.value_of("filename").unwrap_or("-").to_string();

    let conditions = filter::FilterConditions {
        include_terms: match app.values_of("include_term") {
            Some(values) => Some(values.map(|v| v.to_string()).collect()),
            None => None,
        },
        exclude_terms: match app.values_of("exclude_term") {
            Some(values) => Some(values.map(|v| v.to_string()).collect()),
            None => None,
        },
        latest_time: match app.value_of("time_filter_minutes") {
            Some(minutes) => {
                Some(Duration::minutes(minutes.parse().expect("Minutes value must be numeric")))
            }
            None => None,
        },
    };

    let graphite_server = match app.value_of("graphite-server") {
        Some(value) => Some(String::from(value)),
        None => None,
    };

    let graphite_port: Option<u16> = match app.value_of("graphite-port") {
        Some(value) => Some(value.parse().expect("Port number must be numeric.")),
        None => None,
    };

    let graphite_prefix = match app.value_of("graphite-prefix") {
        Some(value) => Some(String::from(value)),
        None => None,
    };

    Ok(RequestLogAnalyzerArgs {
        filename: filename,
        conditions: conditions,
        graphite_server: graphite_server,
        graphite_port: graphite_port,
        graphite_prefix: graphite_prefix,
    })
}

#[cfg(test)]
mod tests {
    use filter;
    use chrono::*;
    use super::*;

    #[test]
    fn test_parse_args_default() {
        let raw_args = vec!["request_log_analyzer".to_string()];

        let expected = RequestLogAnalyzerArgs {
            filename: String::from("-"),
            conditions: filter::FilterConditions {
                include_terms: None,
                exclude_terms: None,
                latest_time: None,
            },
            graphite_server: None,
            graphite_port: Some(2003),
            graphite_prefix: None,
        };

        let result = parse_args(raw_args).unwrap();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_args_all() {
        let raw_args = vec![String::from("request_log_analyzer"),
                            String::from("--include"), String::from("one"),
                            String::from("--exclude"), String::from("this other"),
                            String::from("-t"), String::from("10"),
                            String::from("my-logfile.log"),
                            String::from("--graphite-server"), String::from("localhost"),
                            String::from("--graphite-port"), String::from("4000"),
                            String::from("--graphite-prefix"), String::from("prod"),
                            ];

        let expected = RequestLogAnalyzerArgs {
            filename: String::from("my-logfile.log"),
            conditions: filter::FilterConditions {
                include_terms: Some(vec![String::from("one")]),
                exclude_terms: Some(vec![String::from("this other")]),
                latest_time: Some(Duration::minutes(10)),
            },
            graphite_server: Some(String::from("localhost")),
            graphite_port: Some(4000),
            graphite_prefix: Some(String::from("prod")),
        };

        let result = parse_args(raw_args).unwrap();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_args_mutliple_include_exclude() {
        let raw_args = vec![String::from("request_log_analyzer"),
                            String::from("--include"), String::from("one"),
                            String::from("--include"), String::from("two"),
                            String::from("--exclude"), String::from("this other"),
                            String::from("--exclude"), String::from("more"),
                            String::from("my-logfile.log"),
                            ];

        let expected = RequestLogAnalyzerArgs {
            filename: String::from("my-logfile.log"),
            conditions: filter::FilterConditions {
                include_terms: Some(vec![String::from("one"),
                                         String::from("two")]),
                exclude_terms: Some(vec![String::from("this other"),
                                         String::from("more")]),
                latest_time: None,
            },
            graphite_server: None,
            graphite_port: Some(2003),
            graphite_prefix: None,
        };

        let result = parse_args(raw_args).unwrap();

        assert_eq!(result, expected);
    }
}
