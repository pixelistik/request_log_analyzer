[![Build Status](https://travis-ci.org/pixelistik/request_log_analyzer.svg?branch=master)](https://travis-ci.org/pixelistik/request_log_analyzer)

## Installation
Download the Linux x64 binary `request_log_analyzer` from
the [releases page](https://github.com/pixelistik/request_log_analyzer/releases/latest)
(built on Travis CI servers).

Alternatively you can build from source, if you have a Rust toolchain set up:
Clone the repository and run `cargo build`.

## Usage

    Request.log Analyzer

    USAGE:
        request_log_analyzer [OPTIONS] [--] [<FILE>]

    FLAGS:
        -h, --help       Prints help information
        -V, --version    Prints version information

    OPTIONS:
            --exclude <TERM>...                    Exclude lines that contain one of these terms
            --graphite-port <GRAPHITE_PORT>         [default: 2003]
            --graphite-prefix <GRAPHITE_PREFIX>    Prefix for Graphite key, e.g. 'servers.prod.publisher1'
            --graphite-server <GRAPHITE_SERVER>    Send values to this Graphite server instead of stdout
            --include <TERM>...                    Only include lines that contain one of these terms
        -t <MINUTES>                               Limit to the last n minutes

    ARGS:
        <FILE>    Log file to analyze, defaults to stdin

## Example output
    $ request_log_analyzer crx-quickstart/logs/request.log
    count:	54510
    time.avg:	127
    time.min:	1
    time.median:	6
    time.90percent:	27
    time.max:	15747
