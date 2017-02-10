use std::mem;
use std::marker::PhantomData;

pub use applied::interval::{Interval, KeyInterval};

pub use self::plain::{TeardownTreeMap, TeardownTreeSet};
pub use self::interval::{IntervalTeardownTreeMap, IntervalTeardownTreeSet};
pub use base::{TeardownTreeRefill, Sink, Entry};


pub trait TreeWrapperAccess {
    type Repr;
    type Wrapper;

    fn internal(&self) -> &Self::Wrapper;
    fn internal_mut(&mut self) -> &mut Self::Wrapper;
    fn into_internal(self) -> Self::Wrapper;
    fn from_internal(wrapper: Self::Wrapper) -> Self;
    fn from_repr(repr: Self::Repr) -> Self;
}



mod plain {
    use base::{TreeRepr, TeardownTreeRefill, Sink, Entry, Key, ItemFilter};
    use applied::plain_tree::{PlTree, PlNode};
    use super::SinkAdapter;

    use std::fmt;
    use std::fmt::{Debug, Display, Formatter};
    use std::ops::Range;
    use std::mem;


    #[derive(Clone)]
    pub struct TeardownTreeMap<K: Ord+Clone, V> {
        internal: PlTree<K,V>
    }

    impl<K: Ord+Clone, V> TeardownTreeMap<K, V> {
        /// Creates a new `TeardownTreeMap` with the given set of items. The items can be given in
        /// any order. Duplicate keys are allowed and supported.
        #[inline] pub fn new(mut items: Vec<(K, V)>) -> TeardownTreeMap<K, V> {
            items.sort_by(|a, b| a.0.cmp(&b.0));
            Self::with_sorted(items)
        }

        /// Creates a new `TeardownTreeMap` with the given set of items. Duplicate keys are allowed
        /// and supported.
        /// **Note**: the items are assumed to be sorted!
        #[inline] pub fn with_sorted(sorted: Vec<(K, V)>) -> TeardownTreeMap<K, V> {
            TeardownTreeMap { internal: PlTree::with_sorted(sorted) }
        }

        /// Returns true if the map contains the given key.
        #[inline] pub fn contains_key(&self, search: &K) -> bool {
            self.internal.contains(search)
        }

        /// Executes a range query.
        #[inline] pub fn query_range<'a, S: Sink<&'a Entry<K, V>>>(&'a self, range: Range<K>, sink: &mut S) {
            self.internal.query_range(range, sink)
        }

        /// Deletes the item with the given key from the tree and returns it (or None).
        #[inline] pub fn delete(&mut self, search: &K) -> Option<V> {
            self.internal.delete(search)
        }

        /// Deletes all items inside the half-open `range` from the tree and stores them in the output
        /// Vec. The items are returned in order.
        #[inline] pub fn delete_range(&mut self, range: Range<K>, output: &mut Vec<(K, V)>) {
            self.internal.delete_range(range, output)
        }

        /// Deletes all items inside the half-open `range` from the tree for which filter.accept() returns
        /// true and stores them in the output Vec. The items are returned in order.
        #[inline] pub fn filter_range<Flt>(&mut self, range: Range<K>, filter: Flt, output: &mut Vec<(K, V)>)
            where Flt: ItemFilter<K>
        {
            self.internal.filter_range(range, filter, output)
        }

        /// Deletes all items inside the half-open `range` from the tree and stores them in the output Vec.
        #[inline] pub fn delete_range_ref(&mut self, range: Range<&K>, output: &mut Vec<(K, V)>) {
            self.internal.delete_range_ref(range, output)
        }

        /// Deletes all items inside the half-open `range` from the tree for which filter.accept() returns
        /// true and stores them in the output Vec. The items are returned in order.
        #[inline] pub fn filter_range_ref<Flt>(&mut self, range: Range<&K>, filter: Flt, output: &mut Vec<(K, V)>)
            where Flt: ItemFilter<K>
        {
            self.internal.filter_range_ref(range, filter, output)
        }

        /// Returns the number of items in this tree.
        #[inline] pub fn size(&self) -> usize { self.internal.size() }

        #[inline] pub fn is_empty(&self) -> bool { self.size() == 0 }

        /// Removes all items from the tree (the items are dropped, but the internal storage is not).
        #[inline] pub fn clear(&mut self) { self.internal.clear(); }
    }

    impl<K: Ord+Clone+Debug, V> Debug for TeardownTreeMap<K, V> {
        fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
            Debug::fmt(&self.internal, fmt)
        }
    }

    impl<K: Ord+Clone+Debug, V> Display for TeardownTreeMap<K, V> {
        fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
            Display::fmt(&self.internal, fmt)
        }
    }

    impl<K: Ord+Clone+Copy, V: Copy> TeardownTreeRefill for TeardownTreeMap<K, V> {
        fn refill(&mut self, master: &Self) {
            self.internal.refill(&master.internal)
        }
    }


    impl<K: Ord+Clone, V> super::TreeWrapperAccess for TeardownTreeMap<K, V> {
        type Repr = TreeRepr<PlNode<K,V>>;
        type Wrapper = PlTree<K,V>;

        fn internal(&self) -> &PlTree<K,V> {
            &self.internal
        }

        fn internal_mut(&mut self) -> &mut Self::Wrapper {
            &mut self.internal
        }

        fn into_internal(self) -> PlTree<K, V> {
            self.internal
        }

        fn from_internal(wrapper: PlTree<K, V>) -> Self {
            TeardownTreeMap { internal: wrapper }
        }

        fn from_repr(repr: Self::Repr) -> Self {
            Self::from_internal(PlTree::with_repr(repr))
        }
    }


    #[derive(Clone, Debug)]
    pub struct TeardownTreeSet<T: Ord+Clone> {
        map: TeardownTreeMap<T, ()>
    }

    impl<T: Ord+Clone> TeardownTreeSet<T> {
        /// Creates a new `TeardownTreeSet` with the given set of items. The items can be given in any
        /// order. Duplicates are allowed and supported.
        pub fn new(items: Vec<T>) -> TeardownTreeSet<T> {
            let map_items = super::conv_to_tuple_vec(items);
            TeardownTreeSet { map: TeardownTreeMap::new(map_items) }
        }

        /// Creates a new `TeardownTreeSet` with the given set of items. Duplicates are allowed and
        /// supported.
        /// **Note**: the items are assumed to be sorted!
        pub fn with_sorted(sorted: Vec<T>) -> TeardownTreeSet<T> {
            let map_items = super::conv_to_tuple_vec(sorted);
            TeardownTreeSet { map: TeardownTreeMap::with_sorted(map_items) }
        }

        /// Returns true if the set contains the given item.
        pub fn contains(&self, search: &T) -> bool {
            self.map.contains_key(search)
        }

        /// Executes a range query.
        pub fn query_range<'a, S: Sink<&'a T>>(&'a self, range: Range<T>, sink: &mut S) {
            self.map.query_range(range, &mut SinkAdapter::new(sink))
        }

        /// Deletes the item with the given key from the tree and returns it (or None).
        pub fn delete(&mut self, search: &T) -> bool {
            self.map.delete(search).is_some()
        }

        /// Deletes all items inside the half-open `range` from the tree and stores them in the output
        /// Vec. The items are returned in order.
        pub fn delete_range(&mut self, range: Range<T>, output: &mut Vec<T>) {
            let map_output = unsafe { mem::transmute(output) };
            self.map.delete_range(range, map_output)
        }

        /// Deletes all items inside the half-open `range` from the tree for which filter.accept() returns
        /// true and stores them in the output Vec. The items are returned in order.
        pub fn filter_range<Flt>(&mut self, range: Range<T>, filter: Flt, output: &mut Vec<T>)
            where Flt: ItemFilter<T>
        {
            let map_output = unsafe { mem::transmute(output) };
            self.map.filter_range(range, filter, map_output)
        }

        /// Deletes all items inside the half-open `range` from the tree and stores them in the output Vec.
        pub fn delete_range_ref(&mut self, range: Range<&T>, output: &mut Vec<T>) {
            let map_output = unsafe { mem::transmute(output) };
            self.map.delete_range_ref(range, map_output)
        }

        /// Deletes all items inside the half-open `range` from the tree for which filter.accept() returns
        /// true and stores them in the output Vec. The items are returned in order.
        pub fn filter_range_ref<Flt>(&mut self, range: Range<&T>, filter: Flt, output: &mut Vec<T>)
            where Flt: ItemFilter<T>
        {
            let map_output = unsafe { mem::transmute(output) };
            self.map.filter_range_ref(range, filter, map_output)
        }

        /// Returns the number of items in this tree.
        pub fn size(&self) -> usize { self.map.size() }

        pub fn is_empty(&self) -> bool { self.map.is_empty() }

        /// Removes all items from the tree (the items are dropped, but the internal storage is not).
        pub fn clear(&mut self) { self.map.clear(); }
    }

    impl<K: Ord+Clone+Copy> TeardownTreeRefill for TeardownTreeSet<K> {
        fn refill(&mut self, master: &Self) {
            self.map.refill(&master.map)
        }
    }

    impl<K: Key> super::TreeWrapperAccess for TeardownTreeSet<K> {
        type Repr = TreeRepr<PlNode<K, ()>>;
        type Wrapper = PlTree<K, ()>;

        fn internal(&self) -> &PlTree<K,()> {
            &self.map.internal
        }

        fn internal_mut(&mut self) -> &mut PlTree<K,()> {
            &mut self.map.internal
        }

        fn into_internal(self) -> PlTree<K, ()> {
            self.map.internal
        }

        fn from_internal(wrapper: PlTree<K, ()>) -> Self {
            TeardownTreeSet { map: TeardownTreeMap { internal: wrapper } }
        }

        fn from_repr(repr: Self::Repr) -> Self {
            Self::from_internal(PlTree::with_repr(repr))
        }
    }

    impl<T: Ord+Clone+Debug> Display for TeardownTreeSet<T> {
        fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
            Display::fmt(&self.map, fmt)
        }
    }
}



mod interval {
    use std::mem;
    use std::fmt;
    use std::fmt::{Debug, Display, Formatter};

    use base::{TreeRepr, TeardownTreeRefill, ItemFilter, Entry, Sink};
    use super::SinkAdapter;

    use applied::AppliedTree;
    use applied::interval::{Interval, IvNode};
    use applied::interval_tree::{IvTree};

    #[derive(Clone)]
    pub struct IntervalTeardownTreeMap<Iv: Interval, V> {
        internal: IvTree<Iv, V>
    }

    impl<Iv: Interval, V> IntervalTeardownTreeMap<Iv, V> {
        /// Creates a new `IntervalTeardownTreeMap` with the given set of intervals. The items can be
        /// given in any order. Duplicates are allowed and supported.
        pub fn new(mut items: Vec<(Iv, V)>) -> IntervalTeardownTreeMap<Iv, V> {
            items.sort_by(|a, b| a.0.cmp(&b.0));
            Self::with_sorted(items)
        }

        /// Creates a new `IntervalTeardownTreeMap` with the given set of intervals. Duplicates are
        /// allowed and supported.
        /// **Note**: the items are assumed to be sorted with respect to `Interval::cmp()`!
        pub fn with_sorted(sorted: Vec<(Iv, V)>) -> IntervalTeardownTreeMap<Iv, V> {
            IntervalTeardownTreeMap { internal: IvTree::with_sorted(sorted) }
        }

        /// Returns true if the map contains the given key.
        pub fn contains_key(&self, search: &Iv) -> bool {
            self.internal.contains(search)
        }

        /// Executes an overlap query.
        pub fn query_overlap<'a, S: Sink<&'a Entry<Iv, V>>>(&'a self, search: &Iv, sink: &mut S) {
            self.internal.query_overlap(search, sink)
        }

        /// Deletes the item with the given key from the tree and returns it (or None).
        #[inline]
        pub fn delete(&mut self, search: &Iv) -> Option<V> {
            self.internal.delete(search)
        }

        /// Deletes all intervals that overlap with the `search` interval from the tree and stores
        /// them in the output Vec. The items are returned in order.
        #[inline]
        pub fn delete_overlap(&mut self, search: &Iv, output: &mut Vec<(Iv, V)>) {
            self.internal.delete_overlap(search, output)
        }

        /// Deletes all intervals that overlap with the `search` interval and match the filter from
        /// the tree and stores the associated items in the output Vec. The items are returned in order.
        pub fn filter_overlap<Flt>(&mut self, search: &Iv, f: Flt, output: &mut Vec<Iv>)
            where Flt: ItemFilter<Iv>
        {
            let map_output = unsafe { mem::transmute(output) };
            self.internal.filter_overlap(search, f, map_output)
        }

        /// Returns the number of items in this tree.
        pub fn size(&self) -> usize {
            self.internal.size()
        }

        pub fn is_empty(&self) -> bool { self.size() == 0 }

        /// Removes all items from the tree (the items are dropped, but the internal storage is not).
        pub fn clear(&mut self) { self.internal.clear(); }
    }


    impl<Iv: Interval, V> super::TreeWrapperAccess for IntervalTeardownTreeMap<Iv, V> {
        type Repr = TreeRepr<IvNode<Iv,V>>;
        type Wrapper = IvTree<Iv,V>;

        fn internal(&self) -> &IvTree<Iv, V> {
            &self.internal
        }

        fn internal_mut(&mut self) -> &mut IvTree<Iv, V> {
            &mut self.internal
        }

        fn into_internal(self) -> IvTree<Iv, V> {
            self.internal
        }

        fn from_internal(wrapper: IvTree<Iv, V>) -> Self {
            IntervalTeardownTreeMap { internal: wrapper }
        }

        fn from_repr(repr: Self::Repr) -> Self {
            Self::from_internal(IvTree::with_repr(repr))
        }
    }

    impl<Iv: Interval+Copy, V: Copy> TeardownTreeRefill for IntervalTeardownTreeMap<Iv, V> {
        fn refill(&mut self, master: &Self) {
            self.internal.refill(&master.internal)
        }
    }


    #[derive(Clone)]
    pub struct IntervalTeardownTreeSet<Iv: Interval> {
        map: IntervalTeardownTreeMap<Iv, ()>
    }

    impl<Iv: Interval> IntervalTeardownTreeSet<Iv> {
        /// Creates a new `IntervalTeardownTreeSet` with the given set of intervals. The items can be
        /// given in any order. Duplicates are allowed and supported.
        pub fn new(items: Vec<Iv>) -> IntervalTeardownTreeSet<Iv> {
            let map_items = super::conv_to_tuple_vec(items);
            IntervalTeardownTreeSet { map: IntervalTeardownTreeMap::new(map_items) }
        }

        /// Creates a new `IntervalTeardownTreeSet` with the given set of items. Duplicates are allowed
        /// and supported.
        /// **Note**: the items are assumed to be sorted!
        pub fn with_sorted(sorted: Vec<Iv>) -> IntervalTeardownTreeSet<Iv> {
            let map_items = super::conv_to_tuple_vec(sorted);
            IntervalTeardownTreeSet { map: IntervalTeardownTreeMap::with_sorted(map_items) }
        }

        /// Returns true if the set contains the given item.
        pub fn contains(&self, search: &Iv) -> bool {
            self.map.contains_key(search)
        }

        /// Executes an overlap query.
        pub fn query_overlap<'a, S: Sink<&'a Iv>>(&'a self, search: &Iv, sink: &mut S) {
            self.map.query_overlap(search, &mut SinkAdapter::new(sink))
        }

        /// Deletes the given interval from the tree and returns true (or false if it was not found).
        pub fn delete(&mut self, search: &Iv) -> bool {
            self.map.delete(search).is_some()
        }

        /// Deletes all intervals that overlap with the `search` interval from the tree and stores
        /// them in the output Vec. The items are returned in order.
        pub fn delete_overlap(&mut self, search: &Iv, output: &mut Vec<Iv>) {
            let map_output = unsafe { mem::transmute(output) };
            self.map.delete_overlap(search, map_output)
        }

        /// Deletes all intervals that overlap with the `search` interval and match the filter from
        /// the tree and stores them in the output Vec. The items are returned in order.
        pub fn filter_overlap<Flt>(&mut self, search: &Iv, f: Flt, output: &mut Vec<Iv>)
            where Flt: ItemFilter<Iv>
        {
            let map_output = unsafe { mem::transmute(output) };
            self.map.filter_overlap(search, f, map_output)
        }


        /// Returns the number of items in this tree.
        pub fn size(&self) -> usize { self.map.size() }

        pub fn is_empty(&self) -> bool { self.map.is_empty() }

        /// Removes all items from the tree (the items are dropped, but the internal storage is not).
        pub fn clear(&mut self) { self.map.clear(); }
    }

    impl<Iv: Interval> super::TreeWrapperAccess for IntervalTeardownTreeSet<Iv> {
        type Repr = TreeRepr<IvNode<Iv, ()>>;
        type Wrapper = IvTree<Iv, ()>;

        fn internal(&self) -> &IvTree<Iv, ()> {
            &self.map.internal
        }

        fn internal_mut(&mut self) -> &mut IvTree<Iv, ()> {
            &mut self.map.internal
        }

        fn into_internal(self) -> IvTree<Iv, ()> {
            self.map.internal
        }

        fn from_internal(wrapper: IvTree<Iv, ()>) -> Self {
            IntervalTeardownTreeSet { map: IntervalTeardownTreeMap { internal: wrapper } }
        }

        fn from_repr(repr: Self::Repr) -> Self {
            Self::from_internal(IvTree::with_repr(repr))
        }
    }

    impl<Iv: Interval+Copy> TeardownTreeRefill for IntervalTeardownTreeSet<Iv> {
        fn refill(&mut self, master: &Self) {
            self.map.refill(&master.map)
        }
    }


    impl<Iv: Interval+Debug, V> Debug for IntervalTeardownTreeMap<Iv, V> where Iv::K: Debug {
        fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
            Debug::fmt(&self.internal, fmt)
        }
    }

    impl<Iv: Interval, V> Display for IntervalTeardownTreeMap<Iv, V> where Iv::K: Debug {
        fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
            Display::fmt(&self.internal, fmt)
        }
    }

    impl<Iv: Interval+Debug> Debug for IntervalTeardownTreeSet<Iv> where Iv::K: Debug {
        fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
            Debug::fmt(&self.map, fmt)
        }
    }

    impl<Iv: Interval> Display for IntervalTeardownTreeSet<Iv> where Iv::K: Debug {
        fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
            Display::fmt(&self.map, fmt)
        }
    }
}

#[inline(always)]
fn conv_to_tuple_vec<K>(items: Vec<K>) -> Vec<(K, ())> {
    unsafe { mem::transmute(items) }
}



struct SinkAdapter<'a, 'b, T: 'a, S: Sink<&'a T>+'b> {
    sink: &'b mut S,
    _ph: PhantomData<&'a T>
}

impl<'a, 'b, T: 'a, S: Sink<&'a T>+'b> SinkAdapter<'a, 'b, T, S> {
    fn new(sink: &'b mut S) -> Self {
        SinkAdapter { sink: sink, _ph: PhantomData }
    }
}

impl<'a, 'b, T: 'a, S: Sink<&'a T>+'b> Sink<&'a Entry<T, ()>> for SinkAdapter<'a, 'b, T, S> {
    #[inline] fn consume(&mut self, entry: &'a Entry<T, ()>) {
        self.sink.consume(&entry.key)
    }
}
