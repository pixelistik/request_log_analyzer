pub mod terminal;
pub mod graphite;
pub mod prometheus;
pub mod influxdb;

use result;

pub trait Renderer {
    fn render(&mut self, result: result::RequestLogAnalyzerResult) -> ();
}
