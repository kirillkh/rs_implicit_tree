use std::cmp::Ordering;
use std::ops::{Deref, DerefMut};

use base::{Node, Key, KeyVal};


pub trait Interval: Sized+Key {
    type K: Key;

    fn a(&self) -> &Self::K;
    fn b(&self) -> &Self::K;

    fn intersects(&self, other: &Self) -> bool {
        self.a() < other.b() && other.a() < self.b()
            || self.a() == other.a() // interpret empty intervals as points
    }
}

#[derive(Debug, Clone)]
pub struct KeyInterval<K: Key> {
    a: K,
    b: K
}

impl<K: Key> KeyInterval<K> {
    pub fn new(a: K, b: K) -> KeyInterval<K> {
        KeyInterval { a:a, b:b }
    }
}

impl<K: Key> Interval for KeyInterval<K> {
    type K = K;

    fn a(&self) -> &Self::K {
        &self.a
    }

    fn b(&self) -> &Self::K {
        &self.b
    }
}


#[derive(Clone)]
pub struct IvNode<Iv: Interval, V> {
    pub kv: KeyVal<Iv, V>,
    pub maxb: Iv::K
}

impl<Iv: Interval, V> Deref for IvNode<Iv, V> {
    type Target = KeyVal<Iv, V>;

    fn deref(&self) -> &Self::Target {
        &self.kv
    }
}

impl<Iv: Interval, V> DerefMut for IvNode<Iv, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.kv
    }
}


impl<Iv: Interval, V> Node for IvNode<Iv, V> {
    type K = Iv;
    type V = V;

    fn new(key: Iv, val: V) -> Self {
        let maxb = key.b().clone();
        IvNode { kv: KeyVal::new(key, val), maxb:maxb }
    }

    fn into_kv(self) -> KeyVal<Iv, V> {
        self.kv
    }
}

impl<K: Key> PartialEq for KeyInterval<K> {
    fn eq(&self, other: &Self) -> bool {
        self.a() == other.a() && self.b() == other.b()
    }
}
impl<K: Key> Eq for KeyInterval<K> {}

impl<K: Key> PartialOrd for KeyInterval<K> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<K: Key> Ord for KeyInterval<K> {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.a().cmp(other.a()) {
            Ordering::Less => Ordering::Less,
            Ordering::Greater => Ordering::Greater,
            Ordering::Equal => self.b().cmp(other.b())
        }
    }
}
