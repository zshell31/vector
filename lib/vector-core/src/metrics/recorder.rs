use std::sync::Arc;

use metrics::{Counter, Gauge,  Histogram, Key, Recorder, Unit};
use metrics_util::Registry;

/// [`VectorRecorder`] is a [`metrics::Recorder`] implementation that's suitable
/// for the advanced usage that we have in Vector.
pub(crate) struct VectorRecorder {
    registry: Arc<Registry>,
}

impl VectorRecorder {
    pub fn new(registry: Arc<Registry>) -> Self {
        Self { registry }
    }
}

impl Recorder for VectorRecorder {
    fn describe_counter(&self, _: &Key, _: Option<Unit>, _: Option<&'static str>) {}

    fn describe_gauge(&self, _: &Key, _: Option<Unit>, _: Option<&'static str>) {}

    fn describe_histogram(&self, _: &Key, _: Option<Unit>, _: Option<&'static str>) {}

    fn register_counter(&self, key: &Key) -> Counter {
        self.registry.get_or_create_counter(key, |c| Counter::from_arc(c.clone()))
    }

    fn register_gauge(&self, key: &Key) -> Gauge {
        self.registry.get_or_create_gauge(key, |g| Gauge::from_arc(g.clone()))
    }

    fn register_histogram(&self, key: &Key) -> Histogram {
        self.registry.get_or_create_histogram(key, |h| Histogram::from_arc(h.clone()))
    }
}
