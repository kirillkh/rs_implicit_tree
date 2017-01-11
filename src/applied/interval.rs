use std::cmp::Ordering;


pub trait Interval: Sized+Ord {
    type K: Ord+Clone;

    fn a(&self) -> &Self::K;
    fn b(&self) -> &Self::K;

    fn intersects(&self, other: &Self) -> bool {
        self.a() < other.b() && other.a() < self.b()
            || self.a() == other.a() // interpret empty intervals as points
    }
}

#[derive(Debug, Clone)]
pub struct KeyInterval<K: Ord+Clone> {
    a: K,
    b: K
}

impl<K: Ord+Clone> KeyInterval<K> {
    pub fn new(a: K, b: K) -> KeyInterval<K> {
        KeyInterval { a:a, b:b }
    }
}


impl<K: Ord+Clone> Interval for KeyInterval<K> {
    type K = K;

    fn a(&self) -> &Self::K {
        &self.a
    }

    fn b(&self) -> &Self::K {
        &self.b
    }
}

#[derive(Clone)]
pub struct AugValue<Iv: Interval, V> {
    pub maxb: Iv::K,
    pub val: V
}

impl<Iv: Interval, V> AugValue<Iv, V> {
    #[inline] pub fn new(maxb: Iv::K, val: V) -> AugValue<Iv, V> {
        AugValue { maxb: maxb, val: val }
    }

    #[inline] pub fn maxb(&self) -> &Iv::K {
        &self.maxb
    }

//    #[inline] pub fn val(&self) -> &V {
//        &self.val
//    }
}

impl<K: Ord+Clone> PartialEq for KeyInterval<K> {
    fn eq(&self, other: &Self) -> bool {
        self.a() == other.a() && self.b() == other.b()
    }
}
impl<K: Ord+Clone> Eq for KeyInterval<K> {}

impl<K: Ord+Clone> PartialOrd for KeyInterval<K> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<K: Ord+Clone> Ord for KeyInterval<K> {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.a().cmp(other.a()) {
            Ordering::Less => Ordering::Less,
            Ordering::Greater => Ordering::Greater,
            Ordering::Equal => self.b().cmp(other.b())
        }
    }
}
