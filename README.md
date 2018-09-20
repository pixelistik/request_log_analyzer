[![Build Status](https://travis-ci.org/pixelistik/request_log_analyzer.svg?branch=master)](https://travis-ci.org/pixelistik/request_log_analyzer)
[![codecov](https://codecov.io/gh/pixelistik/request_log_analyzer/branch/master/graph/badge.svg)](https://codecov.io/gh/pixelistik/request_log_analyzer)

## Installation
Download and unzip one of the 64-bit Linux / macOS / Windows binaries from
the [releases page](https://github.com/pixelistik/request_log_analyzer/releases/latest)
(built on Travis/Appveyor servers). The program is a single binary that you run in
the terminal.

Alternatively you can build from source, if you have a Rust toolchain set up:
Clone the repository and run `cargo build --release`.

## Usage

    Request.log Analyzer

    USAGE:
        request_log_analyzer [OPTIONS] [--] [FILES]...

    FLAGS:
        -h, --help       Prints help information
        -q, --quiet      Don't output results to stdout
        -V, --version    Prints version information

    OPTIONS:
        --exclude <TERM>...                          Exclude lines that contain one of these terms
        --graphite-port <GRAPHITE_PORT>               [default: 2003]
        --graphite-prefix <GRAPHITE_PREFIX>
        Prefix for Graphite key, e.g. 'servers.prod.publisher1'

        --graphite-server <GRAPHITE_SERVER>
        Send values to this Graphite server instead of stdout

        --include <TERM>...
        Only include lines that contain one of these terms

        --influxdb-write-url <INFLUXDB_WRITE_URL>
        base URL of InfluxDB to send metrics to, e.g. 'http://localhost:8086/write?db=mydb'

        --influxdb-tags <INFLUXDB_TAGS>
        tags for the submitted measurement, e.g. 'host=prod3' or 'host=prod3,type=worker'

        --prometheus-listen <BINDING_ADDRESS>
        Address and port to bind Prometheus HTTP server to, e.g. 'localhost:9898'

        -t <MINUTES>                                     Limit to the last n minutes

    ARGS:
        <FILES>...    Log files to analyze, defaults to stdin

## Example output
    $ request_log_analyzer crx-quickstart/logs/request.log
    count:	54510
    time.avg:	127
    time.min:	1
    time.median:	6
    time.90percent:	27
    time.99percent:	3614
    time.max:	15747
    error.client_error_4xx_rate:	0.023
    error.server_error_5xx_rate:	0.0002

## Getting started

### Analyze an entire log file

Run the tool on a request.log file

	$ request_log_analyzer crx-quickstart/logs/request.log
	count:  1221
	time.avg:       840
	time.min:       0
	time.median:    841
	time.90percent: 1537
	time.99percent: 1650
	time.max:       1709
    error.client_error_4xx_rate:	0.02
    error.server_error_5xx_rate:	0.01

Let's run through the results:

There are 1221 request/response pairs in the log file.  
The average response time was 840ms.  
The fastest response was 0ms.  
The median response time was 841ms.  
The 90 percentile response time was 1537ms. That means that 90% of all requests were finished after 1537ms.  
The 99 percentile response time was 1650ms. That means that 99% of all requests were finished after 1650ms.  
The slowest response was 1709ms.  
2% of all requests have failed with a 4xx HTTP error code (client error).  
1% of all requests have failed with a 5xx HTTP error code (server error).  

### Include only certain requests

Let's say we only care about the rendering of HTML pages, so we want to ignore anything else.

	$ request_log_analyzer --include "text/html" crx-quickstart/logs/request.log

Now the result only refers to data where either request or response line contains the specified `--include` term, in this case the MIME type "text/html"

### Restrict to latest period

Now we specifically want to look at the latest hour, because we suspect a recent problem:

	$ request_log_analyzer -t 60 crx-quickstart/logs/request.log

With the `-t` param, only the most recent _n_ minutes will be taken into account.

### Combine everything

	$ request_log_analyzer --include "text/html" \
		--include "content/dam/" \
		--exclude "POST" \
		-t 180 \
		crx-quickstart/logs/request.log

We look at request/resonse lines that contain "text/html" (the MIME type) or a path from the DAM, but we exclude POST requests. Also, we are only interested in the latest 3 hours.

### Piped log data

If the built-in filtering options are not enough, we can use other tools for filtering the log lines and the pipe them into the tool for analysis:

	$ grep "09:..:.." crx-quickstart/logs/request.log | request_log_analyzer

Here we only look at the request/response lines from a specific hour.

## Continuous monitoring

### Graphite

In the examples above, we used `request_log_analyzer` for individual insights.

But it can also be used to continuously feed data into a Graphite, InfluxDB or Prometheus data store, in order to be used for monitoring.

	$ request_log_analyzer -t 5 \
		--graphite-server localhost \
		--graphite-prefix my-app.production.5min \
		crx-quickstart/logs/request.log

This will analyze the latest 5 minutes of the log file, then push it to a Graphite server running on localhost, storing the results under the keys

	my-app.production.5min.requests.count
	my-app.production.5min.requests.time.max
	my-app.production.5min.requests.time.min

etc.

If you set this command up as a cronjob to run every 1 minute, you can constantly monitor the data for the previous 5 minute window.

### InfluxDB

    $ request_log_analyzer -t 5 \
        --quiet \
        --influxdb-write-url "http://localhost:8086/write?db=metrics" \
        --influxdb-tags type=publisher,time=5min \
        crx-quickstart/logs/request.log

This will analyze the latest 5 minutes of the log file, then push it to an InfluxDB server running on localhost, storing the results in the database `metrics` as the measurement `request_log` with the fields `time_max`, `time_min` etc. The result
will not be displayed in the terminal.

`--influxdb-tags` are an optional way to identify and categorize your measurements, in this example the tags `type=publisher` and `time=5min` are set.

### Prometheus

	$ request_log_analyzer -t 5 --prometheus-listen localhost:9898 crx-quickstart/logs/request.log

This will start a Prometheus endpoint (a small HTTP server) on port 9898. Whenever the Prometheus server queries this endpoint, the latest 5 minutes of the `request.log` will be analyzed and the results will be provided under the keys

	request_count
	request_time_max
	request_time_min

etc. If you set up your Prometheus server to pull data from this endpoint, you can constantly monitor the data for the previous 5 minute window.
