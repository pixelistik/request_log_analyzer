use std::io::{self, Read, Write, Cursor};
use std::net::Shutdown;
use std::time::Duration;
use std::cell::Cell;
use std::net::SocketAddr;
use hyper::net::NetworkStream;

// MockStream is copied from Hyper tests
// https://github.com/hyperium/hyper/blob/0.10.x/src/mock.rs
// MIT licensed, Copyright (c) 2014 Sean McArthur
#[derive(Clone, Debug)]
pub struct MockStream {
    pub read: Cursor<Vec<u8>>,
    next_reads: Vec<Vec<u8>>,
    pub write: Vec<u8>,
    pub is_closed: bool,
    pub error_on_write: bool,
    pub error_on_read: bool,
    pub read_timeout: Cell<Option<Duration>>,
    pub write_timeout: Cell<Option<Duration>>,
}

impl PartialEq for MockStream {
    fn eq(&self, other: &MockStream) -> bool {
        self.read.get_ref() == other.read.get_ref() && self.write == other.write
    }
}

impl MockStream {
    pub fn new() -> MockStream {
        MockStream::with_input(b"")
    }

    pub fn with_input(input: &[u8]) -> MockStream {
        MockStream::with_responses(vec![input])
    }

    pub fn with_responses(mut responses: Vec<&[u8]>) -> MockStream {
        MockStream {
            read: Cursor::new(responses.remove(0).to_vec()),
            next_reads: responses.into_iter().map(|arr| arr.to_vec()).collect(),
            write: vec![],
            is_closed: false,
            error_on_write: false,
            error_on_read: false,
            read_timeout: Cell::new(None),
            write_timeout: Cell::new(None),
        }
    }
}

impl Read for MockStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.error_on_read {
            Err(io::Error::new(io::ErrorKind::Other, "mock error"))
        } else {
            match self.read.read(buf) {
                Ok(n) => {
                    if self.read.position() as usize == self.read.get_ref().len() {
                        if self.next_reads.len() > 0 {
                            self.read = Cursor::new(self.next_reads.remove(0));
                        }
                    }
                    Ok(n)
                }
                r => r,
            }
        }
    }
}

impl Write for MockStream {
    fn write(&mut self, msg: &[u8]) -> io::Result<usize> {
        if self.error_on_write {
            Err(io::Error::new(io::ErrorKind::Other, "mock error"))
        } else {
            Write::write(&mut self.write, msg)
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl NetworkStream for MockStream {
    fn peer_addr(&mut self) -> io::Result<SocketAddr> {
        Ok("127.0.0.1:1337".parse().unwrap())
    }

    fn set_read_timeout(&self, dur: Option<Duration>) -> io::Result<()> {
        self.read_timeout.set(dur);
        Ok(())
    }

    fn set_write_timeout(&self, dur: Option<Duration>) -> io::Result<()> {
        self.write_timeout.set(dur);
        Ok(())
    }

    fn close(&mut self, _how: Shutdown) -> io::Result<()> {
        self.is_closed = true;
        Ok(())
    }
}
