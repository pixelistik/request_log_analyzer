[![Build Status](https://travis-ci.org/pixelistik/request_log_analyzer.svg?branch=master)](https://travis-ci.org/pixelistik/request_log_analyzer)

## Usage
    Request.log Analyzer

    USAGE:
    request_log_analyzer [OPTIONS] [<FILE>]

    FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

    OPTIONS:
        --exclude <TERM>                       Excludes lines that contain this term
        --graphite-port <GRAPHITE_PORT>         [default: 2003]
        --graphite-prefix <GRAPHITE_PREFIX>    Prefix for Graphite key, e.g. 'servers.prod.publisher1'
        --graphite-server <GRAPHITE_SERVER>    Send values to this Graphite server instead of stdout
        --include <TERM>                       Only includes lines that contain this term
    -t <MINUTES>                               Limit to the last n minutes

    ARGS:
    <FILE>    Log file to analyze, defaults to stdin
