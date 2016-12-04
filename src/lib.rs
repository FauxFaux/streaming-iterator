
pub trait StreamingIterator {
    type Item: ?Sized;

    fn advance(&mut self);

    fn get(&self) -> Option<&Self::Item>;

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }

    #[inline]
    fn next(&mut self) -> Option<&Self::Item> {
        self.advance();
        (*self).get()
    }

    #[inline]
    fn by_ref(&mut self) -> &mut Self {
        self
    }

    #[inline]
    fn count(mut self) -> usize
        where Self: Sized
    {
        let mut count = 0;
        while let Some(_) = self.next() {
            count += 1;
        }
        count
    }

    #[inline]
    fn filter<F>(self, f: F) -> Filter<Self, F>
        where Self: Sized,
              F: FnMut(&Self::Item) -> bool
    {
        Filter {
            it: self,
            f: f,
        }
    }

    #[inline]
    fn owned(self) -> Owned<Self>
        where Self: Sized,
              Self::Item: ToOwned
    {
        Owned(self)
    }
}

impl<'a, I: ?Sized> StreamingIterator for &'a mut I
    where I: StreamingIterator
{
    type Item = I::Item;

    #[inline]
    fn advance(&mut self) {
        (**self).advance()
    }

    #[inline]
    fn get(&self) -> Option<&Self::Item> {
        (**self).get()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (**self).size_hint()
    }

    #[inline]
    fn next(&mut self) -> Option<&Self::Item> {
        (**self).next()
    }
}

#[inline]
pub fn convert<I>(it: I) -> Convert<I>
    where I: Iterator
{
    Convert {
        it: it,
        item: None,
    }
}

pub struct Convert<I>
    where I: Iterator
{
    it: I,
    item: Option<I::Item>,
}

impl<I> StreamingIterator for Convert<I>
    where I: Iterator
{
    type Item = I::Item;

    #[inline]
    fn advance(&mut self) {
        self.item = self.it.next();
    }

    #[inline]
    fn get(&self) -> Option<&I::Item> {
        self.item.as_ref()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.it.size_hint()
    }

    #[inline]
    fn count(self) -> usize {
        self.it.count()
    }
}

pub struct Filter<I, F> {
    it: I,
    f: F,
}

impl<I, F> StreamingIterator for Filter<I, F>
    where I: StreamingIterator,
          F: FnMut(&I::Item) -> bool
{
    type Item = I::Item;

    #[inline]
    fn advance(&mut self) {
        while let Some(i) = self.it.next() {
            if (self.f)(i) {
                break;
            }
        }
    }

    #[inline]
    fn get(&self) -> Option<&I::Item> {
        self.it.get()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, self.it.size_hint().1)
    }
}

pub struct Owned<I>(I);

impl<I> Iterator for Owned<I>
where I: StreamingIterator,
      I::Item: Sized + ToOwned
{
    type Item = <I::Item as ToOwned>::Owned;

    #[inline]
    fn next(&mut self) -> Option<<I::Item as ToOwned>::Owned> {
        self.0.next().map(ToOwned::to_owned)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_convert() {
        let items = [0, 1];
        let mut it = convert(items.iter().cloned());
        assert_eq!(it.get(), None);
        assert_eq!(it.get(), None);
        it.advance();
        assert_eq!(it.get(), Some(&0));
        assert_eq!(it.get(), Some(&0));
        it.advance();
        assert_eq!(it.get(), Some(&1));
        assert_eq!(it.get(), Some(&1));
        it.advance();
        assert_eq!(it.get(), None);
        assert_eq!(it.get(), None);
        it.advance();
        assert_eq!(it.get(), None);
        assert_eq!(it.get(), None);
    }

    #[test]
    fn filter() {
        let items = [0, 1, 2, 3];
        let mut it = convert(items.iter().cloned()).filter(|x| x % 2 == 0);
        assert_eq!(it.get(), None);
        assert_eq!(it.get(), None);
        it.advance();
        assert_eq!(it.get(), Some(&0));
        assert_eq!(it.get(), Some(&0));
        it.advance();
        assert_eq!(it.get(), Some(&2));
        assert_eq!(it.get(), Some(&2));
        it.advance();
        assert_eq!(it.get(), None);
        assert_eq!(it.get(), None);
        it.advance();
        assert_eq!(it.get(), None);
        assert_eq!(it.get(), None);
    }

    #[test]
    fn count() {
        let items = [0, 1, 2, 3];
        let it = convert(items.iter());
        assert_eq!(it.count(), 4);
    }

    #[test]
    fn owned() {
        let items = [0, 1];
        let it = convert(items.iter().cloned()).owned();
        assert_eq!(it.collect::<Vec<_>>(), items);
    }
}
