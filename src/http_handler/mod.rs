use args;
use analyzer;
use hyper;
use prometheus::Encoder;
use render;
use render::Renderer;
use run;

struct HttpHandler {
    args: args::RequestLogAnalyzerArgs,
    run: fn(&args::RequestLogAnalyzerArgs) -> Option<analyzer::RequestLogAnalyzerResult>,
}

impl hyper::server::Handler for HttpHandler {
    fn handle(&self, _: hyper::server::Request, mut res: hyper::server::Response) {
        let result = (self.run)(&self.args);

        let mut renderer = render::prometheus::PrometheusRenderer::new();
        renderer.render(result);
        res.headers_mut()
            .set(hyper::header::ContentType(renderer.encoder
                .format_type()
                .parse::<hyper::mime::Mime>()
                .unwrap()));
        res.send(&renderer.buffer).unwrap();
    }
}

pub fn listen_http(args: args::RequestLogAnalyzerArgs, binding_address: &str) {
    let handler = HttpHandler {
        args: args,
        run: run,
    };

    info!("listening addr {:?}", binding_address);
    hyper::server::Server::http(binding_address).unwrap().handle(handler).unwrap();
}
