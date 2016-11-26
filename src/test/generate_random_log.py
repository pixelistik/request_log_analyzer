"""
Generate random log lines to stdout

Example:

    python generate_random_log.py 1000

will generate 1000 request lines and up to 1000 matching response lines.
"""
import sys
from random import randint

try:
    count = int(sys.argv[1])
except IndexError:
    count = 1

for i in range(0, count):
    hour = randint(0, 23)
    minute = randint(0, 59)

    id = randint(1, 99999)
    duration = randint(0, 20000)
    no_response_failure_rate = randint(0, 99)

    print("08/Apr/2016:%d:%d:47 +0200 [%d] -> GET /content/%s/page.html HTTP/1.1" % (hour, minute, id, "some"))
    if not no_response_failure_rate == 0:
        print("08/Apr/2016:%d:%s:48 +0200 [%d] <- 200 text/html %dms" % (hour, minute, id, duration))
