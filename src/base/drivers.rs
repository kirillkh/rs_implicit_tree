use std::ptr;
use std::ops::Range;
use base::{KeyVal};

pub trait TraversalDriver<K: Ord, V> {
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




pub struct RangeRefDriver<'a, K: Ord +'a, V: 'a> {
    range: Range<&'a K>,
    output: &'a mut Vec<(K, V)>
}

impl<'a, K: Ord +'a, V> RangeRefDriver<'a, K, V> {
    pub fn new(range: Range<&'a K>, output: &'a mut Vec<(K, V)>) -> RangeRefDriver<'a, K, V> {
        RangeRefDriver { range:range, output:output }
    }

    pub fn from(&self) -> &'a K {
        self.range.start
    }

    pub fn to(&self) -> &'a K {
        self.range.end
    }
}

impl<'a, K: Ord +'a, V> TraversalDriver<K, V> for RangeRefDriver<'a, K, V> {
    type Decision = RangeDecision;

    #[inline(always)]
    fn decide(&self, x: &K) -> Self::Decision {
        let left = self.from() <= x;
        let right = x < self.to();

        RangeDecision { left: left, right: right }
    }

    #[inline(always)]
    fn output(&mut self) -> &mut Vec<(K, V)> {
        &mut self.output
    }
}


pub struct RangeDriver<'a, K: Ord +'a, V: 'a> {
    range: Range<K>,
    output: &'a mut Vec<(K, V)>
}

impl<'a, K: Ord +'a, V> RangeDriver<'a, K, V> {
    pub fn new(range: Range<K>, output: &'a mut Vec<(K, V)>) -> RangeDriver<K, V> {
        RangeDriver { range:range, output: output }
    }

    pub fn from(&self) -> &K {
        &self.range.start
    }

    pub fn to(&self) -> &K {
        &self.range.end
    }
}

impl<'a, K: Ord +'a, V> TraversalDriver<K, V> for RangeDriver<'a, K, V> {
    type Decision = RangeDecision;

    #[inline(always)]
    fn decide(&self, key: &K) -> Self::Decision {
        let left = self.from() <= key;
        let right = key < self.to();

        RangeDecision { left: left, right: right }
    }

    #[inline(always)]
    fn output(&mut self) -> &mut Vec<(K, V)> {
        &mut self.output
    }
}


#[inline(always)]
pub fn consume_unchecked<K: Ord, V>(output: &mut Vec<(K, V)>, item: KeyVal<K, V>) {
    unsafe {
        let len = output.len();
        debug_assert!(len < output.capacity());
        output.set_len(len + 1);
        let p = output.get_unchecked_mut(len);

        // TODO: optimizer fails here, might want to change to "let tuple: (K,V) = mem::transmute(item)" (but that is not guaranteed to work)
        let KeyVal{key, val} = item;
        ptr::write(p, (key, val));
    }
}


#[inline(always)]
pub fn consume_ptr<K: Ord, V>(output: &mut Vec<(K, V)>, src: *const KeyVal<K, V>) {
    unsafe {
        let len = output.len();
        debug_assert!(len < output.capacity());
        output.set_len(len + 1);
        let p = output.get_unchecked_mut(len);

        // TODO: optimizer fails here, might want to change to "let tuple: (K,V) = mem::transmute(item)" (but that is not guaranteed to work)
        let KeyVal{key, val} = ptr::read(src);
        ptr::write(p, (key, val));
    }
}
