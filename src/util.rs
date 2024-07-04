use std::iter::Iterator;
use std::{collections::HashMap, hash::Hash};

use sodium_rust::{Cell, SodiumCtx, Stream};

pub trait StreamSequenceVec<A>
where
    A: Clone + Send + 'static,
{
    fn sequence(&self, sodium_ctx: &SodiumCtx) -> Stream<Vec<A>>;
}

impl<A> StreamSequenceVec<A> for Vec<Stream<A>>
where
    A: Clone + Send + 'static,
{
    fn sequence(&self, sodium_ctx: &SodiumCtx) -> Stream<Vec<A>> {
        let s_never: Stream<Vec<A>> = sodium_ctx.new_stream();

        self.iter()
            .map(|s_a| s_a.map(|a| vec![a.clone()]))
            .fold(s_never, |s_acc, s_b| {
                s_acc.merge(&s_b, |acc, b| {
                    let mut next = acc.clone();
                    next.extend(b.clone().into_iter());
                    next
                })
            })
    }
}

pub trait StreamSequenceHashMap<Key, A>
where
    Key: Clone + Send + Hash + Eq + 'static + Sync,
    A: Clone + Send + 'static,
{
    fn sequence(&self, sodium_ctx: &SodiumCtx) -> Stream<HashMap<Key, A>>;
}

impl<Key, A> StreamSequenceHashMap<Key, A> for HashMap<Key, Stream<A>>
where
    Key: Clone + Send + Hash + Eq + 'static + Sync,
    A: Clone + Send + 'static,
{
    fn sequence(&self, sodium_ctx: &SodiumCtx) -> Stream<HashMap<Key, A>> {
        let s_never: Stream<HashMap<Key, A>> = sodium_ctx.new_stream();

        self.iter()
            .map(|(k, s_a)| {
                let k = k.clone();
                s_a.map(move |a| {
                    let mut set = HashMap::new();
                    set.insert(k.clone(), a.clone());
                    set
                })
            })
            .fold(s_never, |s_acc, s_b| {
                s_acc.merge(&s_b, |acc, b| {
                    let mut next = acc.clone();
                    next.extend(b.clone().into_iter());
                    next
                })
            })
    }
}

#[cfg(test)]
mod test {
    use std::{
        char,
        collections::HashMap,
        sync::{Arc, Mutex},
    };

    use sodium_rust::{SodiumCtx, StreamSink};

    use crate::util::StreamSequenceVec;

    use super::StreamSequenceHashMap;

    #[test]
    fn stream_sequence_vec() {
        let sodium_ctx = SodiumCtx::new();

        let ssink_a: StreamSink<char> = sodium_ctx.new_stream_sink();
        let ssink_b: StreamSink<char> = sodium_ctx.new_stream_sink();
        let ssink_c: StreamSink<char> = sodium_ctx.new_stream_sink();
        let ssink_d: StreamSink<char> = sodium_ctx.new_stream_sink();

        let vec_s_char = vec![
            ssink_a.stream(),
            ssink_b.stream(),
            ssink_c.stream(),
            ssink_d.stream(),
        ];

        let s_vec_char = vec_s_char.sequence(&sodium_ctx);

        let results = Arc::new(Mutex::new(vec![]));
        let l;
        {
            let results = results.clone();
            l = s_vec_char
                .listen(move |result| results.lock().as_mut().unwrap().push(result.clone()));
        }

        sodium_ctx.transaction(|| {
            ssink_a.send('a');
            ssink_c.send('c');
        });

        sodium_ctx.transaction(|| {
            ssink_c.send('c');
            ssink_a.send('a');
        });

        sodium_ctx.transaction(|| {
            ssink_a.send('a');
            ssink_b.send('h');
            ssink_c.send('o');
            ssink_d.send('y');
        });

        sodium_ctx.transaction(|| {
            ssink_d.send('y');
            ssink_a.send('a');
            ssink_c.send('o');
            ssink_b.send('h');
        });

        l.unlisten();

        {
            let lock = results.lock();
            let results: &Vec<Vec<char>> = lock.as_ref().unwrap();
            assert_eq!(
                vec![
                    vec!['a', 'c'],
                    vec!['a', 'c'],
                    vec!['a', 'h', 'o', 'y'],
                    vec!['a', 'h', 'o', 'y'],
                ],
                *results,
            );
        }
    }

    #[test]
    fn stream_sequence_hash_set() {
        let sodium_ctx = SodiumCtx::new();

        let ssink_a: StreamSink<char> = sodium_ctx.new_stream_sink();
        let ssink_b: StreamSink<char> = sodium_ctx.new_stream_sink();
        let ssink_c: StreamSink<char> = sodium_ctx.new_stream_sink();
        let ssink_d: StreamSink<char> = sodium_ctx.new_stream_sink();

        let mut char_to_s_char = HashMap::new();
        char_to_s_char.insert('a', ssink_a.stream());
        char_to_s_char.insert('b', ssink_b.stream());
        char_to_s_char.insert('c', ssink_c.stream());
        char_to_s_char.insert('d', ssink_d.stream());

        let s_char_to_char = char_to_s_char.sequence(&sodium_ctx);

        let results = Arc::new(Mutex::new(vec![]));
        let l;
        {
            let results = results.clone();
            l = s_char_to_char
                .listen(move |result| results.lock().as_mut().unwrap().push(result.clone()));
        }

        sodium_ctx.transaction(|| {
            ssink_a.send('a');
            ssink_c.send('c');
        });

        sodium_ctx.transaction(|| {
            ssink_c.send('c');
            ssink_a.send('a');
        });

        sodium_ctx.transaction(|| {
            ssink_a.send('a');
            ssink_b.send('h');
            ssink_c.send('o');
            ssink_d.send('y');
        });

        sodium_ctx.transaction(|| {
            ssink_d.send('y');
            ssink_a.send('a');
            ssink_c.send('o');
            ssink_b.send('h');
        });

        l.unlisten();

        {
            let lock = results.lock();
            let results: &Vec<HashMap<char, char>> = lock.as_ref().unwrap();
            let expects = vec![
                vec![('a', 'a'), ('c', 'c')].into_iter().collect(),
                vec![('a', 'a'), ('c', 'c')].into_iter().collect(),
                vec![('a', 'a'), ('b', 'h'), ('c', 'o'), ('d', 'y')]
                    .into_iter()
                    .collect(),
                vec![('a', 'a'), ('b', 'h'), ('c', 'o'), ('d', 'y')]
                    .into_iter()
                    .collect(),
            ];
            assert_eq!(results, &expects)
        }
    }
}

pub trait CellSequenceVec<A>
where
    A: Clone + Send + 'static,
{
    fn sequence(&self, sodium_ctx: &SodiumCtx) -> Cell<Vec<A>>;
}

impl<A> CellSequenceVec<A> for Vec<Cell<A>>
where
    A: Clone + Send + 'static,
{
    fn sequence(&self, sodium_ctx: &SodiumCtx) -> Cell<Vec<A>> {
        let c_init: Cell<Vec<A>> = sodium_ctx.new_cell(vec![]);

        self.iter()
            .map(|c_a| c_a.map(|a| vec![a.clone()]))
            .fold(c_init, |c_acc, c_b| {
                c_acc.lift2(&c_b, |acc, b| {
                    let mut next = acc.clone();
                    next.extend(b.clone().into_iter());
                    next
                })
            })
    }
}

pub trait CellSequenceHashMap<Key, A>
where
    Key: Clone + Send + Hash + Eq + 'static + Sync,
    A: Clone + Send + 'static,
{
    fn sequence(&self, sodium_ctx: &SodiumCtx) -> Cell<HashMap<Key, A>>;
}

impl<Key, A> CellSequenceHashMap<Key, A> for HashMap<Key, Cell<A>>
where
    Key: Clone + Send + Hash + Eq + 'static + Sync,
    A: Clone + Send + 'static,
{
    fn sequence(&self, sodium_ctx: &SodiumCtx) -> Cell<HashMap<Key, A>> {
        let c_init: Cell<HashMap<Key, A>> = sodium_ctx.new_cell(HashMap::new());

        self.iter()
            .map(|(k, c_a)| {
                let k = k.clone();
                c_a.map(move |a| {
                    let mut set = HashMap::new();
                    set.insert(k.clone(), a.clone());
                    set
                })
            })
            .fold(c_init, |c_acc, c_b| {
                c_acc.lift2(&c_b, |acc, b| {
                    let mut next = acc.clone();
                    next.extend(b.clone().into_iter());
                    next
                })
            })
    }
}
