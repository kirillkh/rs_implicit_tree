use std::ptr;
use std::ops::Range;
use base::{Key, Entry};

pub trait TraversalDriver<K: Key, V> {
    type Decision: TraversalDecision;

    #[inline(always)]
    fn decide(&self, key: &K) -> Self::Decision;

    #[inline(always)]
    fn output(&mut self) -> &mut Vec<(K, V)>;
}


pub trait TraversalDecision {
    #[inline] fn left(&self) -> bool;
    #[inline] fn right(&self) -> bool;
    #[inline] fn consume(&self) -> bool;
}




#[derive(Clone, Copy, Debug)]
pub struct RangeDecision {
    pub left: bool,
    pub right: bool,
}

impl TraversalDecision for RangeDecision {
    #[inline] fn left(&self) -> bool {
        self.left
    }

    #[inline] fn right(&self) -> bool {
        self.right
    }

    #[inline] fn consume(&self) -> bool {
        self.left && self.right
    }
}




pub struct RangeRefDriver<'a, K: Key +'a, Q: PartialOrd<K>+'a, V: 'a> {
    range: Range<&'a Q>,
    output: &'a mut Vec<(K, V)>
}

impl<'a, K: Key +'a, Q: PartialOrd<K>, V> RangeRefDriver<'a, K, Q, V> {
    pub fn new(range: Range<&'a Q>, output: &'a mut Vec<(K, V)>) -> RangeRefDriver<'a, K, Q, V> {
        RangeRefDriver { range:range, output:output }
    }

    pub fn from(&self) -> &'a Q {
        self.range.start
    }

    pub fn to(&self) -> &'a Q {
        self.range.end
    }
}

impl<'a, K: Key +'a, Q: PartialOrd<K>, V> TraversalDriver<K, V> for RangeRefDriver<'a, K, Q, V> {
    type Decision = RangeDecision;

    #[inline(always)]
    fn decide(&self, x: &K) -> Self::Decision {
        let left = self.from() <= x;
        let right = self.to() > x;

        RangeDecision { left: left, right: right }
    }

    #[inline(always)]
    fn output(&mut self) -> &mut Vec<(K, V)> {
        &mut self.output
    }
}


pub struct RangeDriver<'a, K: Key +'a, Q: PartialOrd<K>, V: 'a> {
    range: Range<Q>,
    output: &'a mut Vec<(K, V)>
}

impl<'a, K: Key +'a, Q: PartialOrd<K>, V> RangeDriver<'a, K, Q, V> {
    pub fn new(range: Range<Q>, output: &'a mut Vec<(K, V)>) -> RangeDriver<K, Q, V> {
        RangeDriver { range:range, output: output }
    }

    pub fn from(&self) -> &Q {
        &self.range.start
    }

    pub fn to(&self) -> &Q {
        &self.range.end
    }
}

impl<'a, K: Key +'a, Q: PartialOrd<K>, V> TraversalDriver<K, V> for RangeDriver<'a, K, Q, V> {
    type Decision = RangeDecision;

    #[inline(always)]
    fn decide(&self, key: &K) -> Self::Decision {
        let left = self.from() <= key;
        let right = self.to() > key || self.from() == key;

        RangeDecision { left: left, right: right }
    }

    #[inline(always)]
    fn output(&mut self) -> &mut Vec<(K, V)> {
        &mut self.output
    }
}


#[inline(always)]
pub fn consume_unchecked<K: Key, V>(output: &mut Vec<(K, V)>, item: Entry<K, V>) {
    unsafe {
        let len = output.len();
        debug_assert!(len < output.capacity());
        output.set_len(len + 1);
        let p = output.get_unchecked_mut(len);

        let entry: (K, V) = item.into();
        ptr::write(p, entry);
    }
}


#[inline(always)]
pub fn consume_ptr<K: Key, V>(output: &mut Vec<(K, V)>, src: *const Entry<K, V>) {
    unsafe {
        let len = output.len();
        debug_assert!(len < output.capacity());
        output.set_len(len + 1);
        let p = output.get_unchecked_mut(len);

        // TODO: optimizer fails here, might want to change to "let tuple: (K,V) = mem::transmute(item)" (but that is not guaranteed to work)
        let Entry {key, val} = ptr::read(src);
        ptr::write(p, (key, val));
    }
}
