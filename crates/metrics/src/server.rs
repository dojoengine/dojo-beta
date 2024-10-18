use core::fmt;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response};

use crate::process::{collect_memory_stats, describe_memory_stats};
use crate::{BoxedHook, Error, Exporter, Hooks};

pub struct Server<MetricsExporter> {
    hooks: Hooks,
    exporter: MetricsExporter,
}

impl<MetricsExporter> Server<MetricsExporter>
where
    MetricsExporter: Exporter + 'static,
{
    /// Creates a new metrics server using the given exporter.
    pub fn new(exporter: MetricsExporter) -> Self {
        describe_memory_stats();
        let hooks: Hooks = vec![Box::new(collect_memory_stats)];
        Self { exporter, hooks }
    }

    pub fn hooks<I: IntoIterator<Item = BoxedHook<()>>>(mut self, hooks: I) -> Self {
        self.hooks.extend(hooks);
        self
    }

    /// Starts an endpoint at the given address to serve Prometheus metrics.
    pub async fn start(self, addr: SocketAddr) -> Result<(), Error> {
        let hooks = Arc::new(move || self.hooks.iter().for_each(|hook| hook()));

        hyper::Server::try_bind(&addr)
            .map_err(|_| Error::FailedToBindAddress { addr })?
            .serve(make_service_fn(move |_| {
                let hook = Arc::clone(&hooks);
                let exporter = self.exporter.clone();
                async move {
                    Ok::<_, Infallible>(service_fn(move |_: Request<Body>| {
                        // call the hooks to collect metrics before exporting them
                        (hook)();
                        // export the metrics from the installed exporter and send as response
                        let metrics = Body::from(exporter.export());
                        async move { Ok::<_, Infallible>(Response::new(metrics)) }
                    }))
                }
            }))
            .await?;

        Ok(())
    }
}

impl<MetricsExporter> fmt::Debug for Server<MetricsExporter>
where
    MetricsExporter: fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Server").field("hooks", &"...").field("exporter", &self.exporter).finish()
    }
}
