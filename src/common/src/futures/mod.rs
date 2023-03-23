use std::{
    collections::BTreeSet,
    pin::Pin,
    task::{Context, Poll},
};

use futures_util::{future::Either, Future, FutureExt};

use crate::collections::lang;

pub fn join_all<'a, T, F: Fn(T)>(
    cx: &mut Context<'_>,
    fut_list: &mut Vec<Pin<Box<dyn Future<Output = T> + Send + 'a>>>,
    callback: F,
) {
    let mut ready_index = BTreeSet::default();
    if fut_list.is_empty() {
        return;
    }
    while let false = lang::index_all_match_mut(fut_list, |idx, fut| {
        if ready_index.contains(&idx) {
            return true;
        }
        match fut.poll_unpin(cx) {
            std::task::Poll::Ready(val) => {
                callback(val);
                ready_index.insert(idx);
                true
            }
            std::task::Poll::Pending => false,
        }
    }) {}
}

pub fn select<Left, Right>(left: Poll<Left>, right: Poll<Right>) -> Poll<Either<Left, Right>> {
    match left {
        Poll::Ready(val) => Poll::Ready(Either::Left(val)),
        Poll::Pending => match right {
            Poll::Ready(val) => Poll::Ready(Either::Right(val)),
            Poll::Pending => Poll::Pending,
        },
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicI64, Ordering};

    use tonic::async_trait;

    use crate::futures::join_all;

    #[tokio::test]
    async fn test_join_all() {
        #[async_trait]
        trait TestJoinAllTrait {
            async fn sum(&self, a: i64, b: i64) -> i64;
        }

        struct TestJoinAllTraitImpl;

        #[async_trait]
        impl TestJoinAllTrait for TestJoinAllTraitImpl {
            async fn sum(&self, a: i64, b: i64) -> i64 {
                a + b
            }
        }

        struct JoinAllSuite;

        impl futures_util::Future for JoinAllSuite {
            type Output = i64;

            fn poll(
                self: std::pin::Pin<&mut Self>,
                cx: &mut std::task::Context<'_>,
            ) -> std::task::Poll<Self::Output> {
                let s = TestJoinAllTraitImpl;
                let ref mut futures = vec![s.sum(1, 2), s.sum(3, 1), s.sum(1, 0), s.sum(4, 2)];
                let r: AtomicI64 = AtomicI64::new(0);

                join_all(cx, futures, |result| {
                    r.fetch_add(result, Ordering::AcqRel);
                });

                std::task::Poll::Ready(r.load(Ordering::Relaxed))
            }
        }

        let handler = tokio::spawn(JoinAllSuite);
        let ok = handler.await;
        assert!(ok.is_ok());
        assert_eq!(ok.unwrap(), 14);
    }
}
