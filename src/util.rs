use sodium_rust::Stream;

pub trait StreamSequence<A>
where
    A: Clone + Send + 'static,
{
    fn sequence(&self) -> Option<Stream<Vec<A>>>;
}

/*
impl<Item, Iter> StreamFromIterableStream for I
where
    Item: Clone + Send + 'static,
    Iter: IntoIterator<Item = Item> + Clone,
{
    fn sequence<B>(&self) -> Option<Stream<B>> {
        let mut iter = self.clone().into_iter();
        let Some(s_first) = iter.next() else {
            return None;
        };

        iter.fold(s_first, f);
        todo!();
    }
}
*/

impl<A> StreamSequence<A> for Vec<Stream<A>>
where
    A: Clone + Send + 'static,
{
    fn sequence(&self) -> Option<Stream<Vec<A>>> {
        let mut iter = self.iter();

        let Some(s_first) = iter.next() else {
            return None;
        };
        let s_vec = s_first.map(|a| vec![a.clone()]);

        let result = iter
            .map(|s_a| s_a.map(|a| vec![a.clone()]))
            .fold(s_vec, |s_acc, s_b| {
                s_acc.merge(&s_b, |acc, b| {
                    let mut next = acc.clone();
                    next.push(b[0].clone());
                    next
                })
            });

        Some(result)
    }
}
