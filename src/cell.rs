use std::sync::Arc;

use sodium_rust::{Cell, Stream};

pub trait StreamWithMorePrimitives<A>
where
    A: Clone + Send + 'static,
{
    fn if_then_else<FN>(&self, cond: FN) -> (Stream<A>, Stream<A>)
    where
        FN: Fn(&A) -> bool + Clone + Send + Sync + 'static;

    fn accum_optional<S, F>(&self, init_state: S, f: F) -> Cell<S>
    where
        S: Clone + Send + 'static,
        F: FnMut(&A, &S) -> Option<S> + Send + Sync + 'static;

    fn accum_filter<S, FCond, FAccum>(&self, init_state: S, cond: FCond, accum: FAccum) -> Cell<S>
    where
        S: Clone + Send + 'static,
        FCond: FnMut(&A, &S) -> bool + Send + Sync + 'static,
        FAccum: FnMut(&A, &S) -> S + Send + Sync + 'static;
}

impl<A> StreamWithMorePrimitives<A> for Stream<A>
where
    A: Clone + Send + 'static,
{
    fn if_then_else<FN>(&self, cond: FN) -> (Stream<A>, Stream<A>)
    where
        FN: Fn(&A) -> bool + Send + Sync + 'static,
    {
        let cond = Arc::new(cond);
        let cond_ = cond.clone();
        let s_then = self.clone().filter(move |a| cond(a));
        let s_else = self.clone().filter(move |a| !cond_(a));

        (s_then, s_else)
    }

    fn accum_optional<S, F>(&self, init_state: S, mut f: F) -> Cell<S>
    where
        S: Clone + Send + 'static,
        F: FnMut(&A, &S) -> Option<S> + Send + Sync + 'static,
    {
        self.accum(init_state, move |a, prev| {
            if let Some(next) = f(a, prev) {
                next
            } else {
                prev.clone()
            }
        })
    }

    fn accum_filter<S, FCond, FAccum>(
        &self,
        init_state: S,
        mut cond: FCond,
        mut accum: FAccum,
    ) -> Cell<S>
    where
        S: Clone + Send + 'static,
        FCond: FnMut(&A, &S) -> bool + Send + Sync + 'static,
        FAccum: FnMut(&A, &S) -> S + Send + Sync + 'static,
    {
        self.accum(init_state, move |a, prev| {
            if cond(a, prev) {
                accum(a, prev)
            } else {
                prev.clone()
            }
        })
    }
}
