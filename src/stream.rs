use futures::{ready, stream::FusedStream, Future, Stream};
use pin_project::pin_project;
use std::{
    pin::Pin,
    task::{Context, Poll},
};

pub(crate) async fn tripwire_handler(closed: bool) {
    futures::future::poll_fn(|_| {
        if closed {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    })
    .await
}

#[pin_project]
pub(crate) struct OnTerminated<S, Fut> {
    #[pin]
    inner: S,
    #[pin]
    on_terminated: Option<Fut>,
}

impl<S, Fut> OnTerminated<S, Fut> {
    fn new(inner: S, on_terminated: Fut) -> Self {
        Self {
            inner,
            on_terminated: Some(on_terminated),
        }
    }
}

impl<S, Fut> Stream for OnTerminated<S, Fut>
where
    S: FusedStream,
    Fut: Future<Output = ()>,
{
    type Item = S::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        let item = this.inner.as_mut().poll_next(cx);
        if this.inner.is_terminated() {
            if let Some(on_terminated) = this.on_terminated.as_mut().as_pin_mut() {
                ready!(on_terminated.poll(cx));
            }
            this.on_terminated.set(None);
        }

        item
    }
}

impl<S, Fut> FusedStream for OnTerminated<S, Fut>
where
    S: FusedStream,
    Fut: Future<Output = ()>,
{
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}

pub(crate) trait StreamUtilExt: Stream
where
    Self: Sized,
{
    fn on_terminated<Fut>(self, on_terminated: Fut) -> OnTerminated<Self, Fut>
    where
        Self: FusedStream,
    {
        OnTerminated::new(self, on_terminated)
    }
}

impl<S: Stream> StreamUtilExt for S {}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::{future::poll_fn, stream::iter, StreamExt};
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };
    use tokio::sync::oneshot;

    #[tokio::test]
    async fn on_terminated() {
        let (trigger, trigger_rx) = oneshot::channel::<()>();
        let mut trigger = Some(trigger);
        let count = Arc::new(AtomicUsize::new(0));
        let count_clone = count.clone();
        let mut s = iter(1..=10).fuse().on_terminated(poll_fn(move |_| {
            trigger.take().map(|trigger| trigger.send(()).unwrap());
            count_clone.fetch_add(1, Ordering::Relaxed);
            Poll::Ready(())
        }));

        let mut items = 0;
        while let Some(_) = s.next().await {
            items += 1;
        }

        assert_eq!(items, 10);
        assert!(trigger_rx.await.is_ok());
        assert_eq!(count.load(Ordering::Relaxed), 1);
        assert!(s.next().await.is_none());
        assert_eq!(count.load(Ordering::Relaxed), 1);
        assert!(s.next().await.is_none());
        assert_eq!(count.load(Ordering::Relaxed), 1);
    }
}
