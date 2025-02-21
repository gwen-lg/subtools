//! duplicate of <https://crates.io/crates/context-iterators> ?

pub trait ContextIterator {
    /// The type of the context of the iterator.
    type Context;
    /// The type of the elements being iterated over.
    type Item;

    fn next(&mut self) -> Option<(&Self::Context, Self::Item)>;

    fn for_each<F>(self, f: F)
    where
        Self: Sized,
        F: FnMut(&Self::Context, Self::Item),
    {
        fn call<C, T>(mut f: impl FnMut(&C, T)) -> impl FnMut(&C, (), T) {
            move |ctx, (), item| f(ctx, item)
        }

        self.fold((), call(f));
    }

    fn fold<B, F>(mut self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(&Self::Context, B, Self::Item) -> B,
    {
        let mut accum = init;
        while let Some((ctx, x)) = self.next() {
            accum = f(ctx, accum, x);
        }
        accum
    }
}

// impl ContextIterator for U {

// }

struct CtxIter<I, T, C>
where
    I: Iterator<Item = T>,
{
    iter: I, // &'a
    context: C,
}

impl<I, T, C> CtxIter<I, T, C>
where
    I: Iterator<Item = T>,
{
    pub const fn new(iter: I, context: C) -> Self {
        // &'a
        Self { iter, context }
    }
    // pub const fn into_new(iter: I, context: C) -> Self {
    //     CtxIter { iter, context }
    // }
}

impl<I, T, C> ContextIterator for CtxIter<I, T, C>
where
    I: Iterator<Item = T>,
{
    type Context = C;
    type Item = T;

    fn next(&mut self) -> Option<(&Self::Context, Self::Item)> {
        let next = self.iter.next();
        next.map(|val| (&self.context, val))
    }

    // fn for_each<F>(mut self, mut f: F)
    // where
    //     Self: Sized,
    //     F: FnMut(&Self::Context, Self::Item),
    // {
    //     while let Some(val) = self.iter.next() {
    //         f(&self.context, val)
    //     }
    // }
}

#[cfg(test)]
mod test {
    use super::{ContextIterator, CtxIter};

    #[test]
    fn test_1() {
        let data = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let x = data.iter();
        let iter = CtxIter::new(x, 5);
        iter.for_each(|ctx, value| {
            println!("Ctx: {ctx} - {value}");
        });
    }
}
