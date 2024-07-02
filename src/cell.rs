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
        FCond: FnMut(&S) -> bool + Send + Sync + 'static,
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

    /// Accumulate on an input event, outputting the new state when cond function returns true.
    ///
    /// As each event is received, the accumulating function `accum` is called with the current state and the new event value.
    /// Next, the condition function `cond` is called with the result of accum function.
    /// If the cond function returns true, the state is updated with the result of the accum function,
    /// else the state is updated with the previous state.
    ///
    /// The accumulating function and condtion function may construct FRP logic or [`Cell::sample`],
    /// in which case it's equivalent to [`snapshot`][Stream::snapshot]ing the cell.
    /// In additon, the function must be referentially transparent.
    ///
    /// ```
    /// let s: Stream<usize> = sodium_ctx.new_stream_sink().stream();
    /// s.accum_filter(
    ///   0,
    ///   |next_state| next_state < 100,
    ///   |event, prev_state| event + prev_state,
    /// )
    /// ```
    fn accum_filter<S, FCond, FAccum>(
        &self,
        init_state: S,
        mut cond: FCond,
        mut accum: FAccum,
    ) -> Cell<S>
    where
        S: Clone + Send + 'static,
        FCond: FnMut(&S) -> bool + Send + Sync + 'static,
        FAccum: FnMut(&A, &S) -> S + Send + Sync + 'static,
    {
        self.accum(init_state, move |a, prev| {
            let next = accum(a, prev);
            if cond(&next) {
                next
            } else {
                prev.clone()
            }
        })
    }
}
