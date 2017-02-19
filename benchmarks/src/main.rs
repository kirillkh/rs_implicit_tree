extern crate libc;
extern crate x86;
extern crate rand;
extern crate treap;
extern crate teardown_tree;
extern crate splay;
//extern crate wio;

use bench_delete_range::{DataMaster, TreapMaster, TreeBulk, TeardownTreeSingle, TeardownSetFilter, IntervalSetFilter, BTreeSetMaster, SplayMaster, IntervalTreeBulk, FilteredIntervalTreeBulk};
use bench_delete_range::{bench_refill_teardown_cycle, bench_refill, imptree_single_elem_range_n, btree_single_delete_n};

use std::time::Duration;

//#[cfg(all(feature = "unstable", target_os = "windows"))]
//fn set_affinity() {
//    assert!(wio::thread::Thread::current().unwrap().set_affinity_mask(8).is_ok());
//}
//
//#[cfg(not(target_os = "windows"))]
//fn set_affinity() {
//}


struct BenchJob<'a> {
    f: &'a Fn(usize, &'a [u64]) -> (String, Vec<u64>),
    spec: &'a [u64]
}

impl<'a> BenchJob<'a> {
    pub fn new(f: &'a Fn(usize, &'a [u64]) -> (String, Vec<u64>), spec: &'a [u64]) -> BenchJob<'a> {
        BenchJob { f: f, spec: spec }
    }
}


fn bench_table(batch_size: usize, action: &str, jobs: &[BenchJob]) {
    println!("\n{:36}, {:10}, {}", "", "", action);
    print!("{:36}, ", "method\\N");
    let ntimings = jobs[0].spec.len();
    let mut n = batch_size;
    for _ in 0..ntimings {
        print!("{:10}, ", n);
        n *= 10;
    }
    println!();

    for job in jobs.iter() {
        let f = job.f;
        let spec = job.spec;

        let (descr, timings) = f(batch_size, spec);

        print!("{:36}, ", descr);
        for time in timings.into_iter() {
            print!("{:10}, ", time);
        }
        println!();
    }
}




fn bench_teardown_full_impl<M: DataMaster>(batch_size: usize, spec: &[u64]) -> (String, Vec<u64>) {
    let mut n = batch_size;
    let timings: Vec<u64> = spec.iter()
        .map(|iters| {
            let time = bench_refill_teardown_cycle::<M>(n, batch_size, *iters);
            n *= 10;
            time
        })
        .collect();

    (M::descr_cycle(), timings)
}


fn bench_refill_impl<M: DataMaster>(_: usize, spec: &[u64]) -> (String, Vec<u64>) {
    let mut n = 10;

    let timings: Vec<u64> = spec.iter()
        .map(|iters| {
            let time = bench_refill::<M>(n, *iters);
            n *= 10;
            time
        })
        .collect();

    (M::descr_cycle(), timings)
}


fn main() {
    bench_table(10, "Refill", &[
        BenchJob::new(&bench_refill_impl::<TreeBulk>,            &[170000000,   80000000,   12000000,   1100000,    65000,  2400,   230]),
        BenchJob::new(&bench_refill_impl::<IntervalTreeBulk>,    &[150000000,   70000000,   11000000,   1000000,    60000,  2200,   210]),
        BenchJob::new(&bench_refill_impl::<TreapMaster>,         &[14000000,     1300000,     60000,      5000,       300,    25,     3]),
        BenchJob::new(&bench_refill_impl::<BTreeSetMaster>,      &[27000000,     3500000,    350000,     30000,      2300,   110,    10]),
        BenchJob::new(&bench_refill_impl::<SplayMaster>,         &[14000000,     1000000,     50000,      4500,       400,    25,     3]),
    ]);


    bench_table(10, "Teardown in bulks of 10 items", &[
        BenchJob::new(&bench_teardown_full_impl::<TreeBulk>,            &[40000000, 3100000,    300000, 12000,  1100,   70, 7]),
        BenchJob::new(&bench_teardown_full_impl::<TeardownTreeSingle>,  &[32000000, 2000000,    120000,  5000,   350,   30, 3]),
        BenchJob::new(&bench_teardown_full_impl::<TeardownSetFilter>,   &[24000000, 2000000,    170000, 10000,  1000,   70, 7]),
        BenchJob::new(&bench_teardown_full_impl::<IntervalTreeBulk>,    &[22000000, 1300000,    100000,  5000,   400,   25, 4]),
        BenchJob::new(&bench_teardown_full_impl::<IntervalSetFilter>,   &[17000000, 1100000,     80000,  5000,   400,   25, 4]),
        BenchJob::new(&bench_teardown_full_impl::<TreapMaster>,         &[ 2500000,  180000,     14000,  1300,    90,    6, 2]),
        BenchJob::new(&bench_teardown_full_impl::<BTreeSetMaster>,      &[13000000,  600000,     32000,  2300,   190,   16, 3]),
        BenchJob::new(&bench_teardown_full_impl::<SplayMaster>,         &[ 3300000,  300000,     24000,  1800,   180,    9, 2]),
    ]);

    bench_table(100, "Teardown in bulks of 100 items", &[
        BenchJob::new(&bench_teardown_full_impl::<TreeBulk>,            &[8000000, 700000, 70000,  4500,   400, 32]),
        BenchJob::new(&bench_teardown_full_impl::<TeardownTreeSingle>,  &[2200000, 150000, 6000,   500,    50,  5]),
        BenchJob::new(&bench_teardown_full_impl::<TeardownSetFilter>,   &[3000000, 270000, 25000,  2000,   180, 28]),
        BenchJob::new(&bench_teardown_full_impl::<IntervalTreeBulk>,    &[6000000, 350000, 35000,  2200,   200, 16]),
        BenchJob::new(&bench_teardown_full_impl::<IntervalSetFilter>,   &[2000000, 200000, 20000,  1500,   150, 16]),
        BenchJob::new(&bench_teardown_full_impl::<TreapMaster>,         &[900000,  50000,  4000,   250,    20,  3]),
        BenchJob::new(&bench_teardown_full_impl::<BTreeSetMaster>,      &[1000000, 50000,  4000,   350,    30,  4]),
        BenchJob::new(&bench_teardown_full_impl::<SplayMaster>,         &[1000000, 50000,  4000,   350,    30,  4]),
    ]);

    bench_table(1000, "Teardown in bulks of 1000 items", &[
        BenchJob::new(&bench_teardown_full_impl::<TreeBulk>,            &[800000, 80000,  8000,   700,    70]),
        BenchJob::new(&bench_teardown_full_impl::<TeardownTreeSingle>,  &[80000,   6000,   500,    60,     6]),
        BenchJob::new(&bench_teardown_full_impl::<TeardownSetFilter>,   &[300000, 25000,  2500,   250,    25]),
        BenchJob::new(&bench_teardown_full_impl::<IntervalTreeBulk>,    &[700000, 60000,  4800,   400,    50]),
        BenchJob::new(&bench_teardown_full_impl::<IntervalSetFilter>,   &[200000, 20000,  2000,   200,    25]),
        BenchJob::new(&bench_teardown_full_impl::<TreapMaster>,         &[50000,  5000,   400,    40,     3]),
        BenchJob::new(&bench_teardown_full_impl::<BTreeSetMaster>,      &[100000, 6000,   600,    40,     4]),
        BenchJob::new(&bench_teardown_full_impl::<SplayMaster>,         &[50000,  6000,   600,    40,     4]),
    ]);


    imptree_single_elem_range_n(100, 100,    200000);
    imptree_single_elem_range_n(1000, 100,   150000);
    imptree_single_elem_range_n(10000, 100,  100000);
    imptree_single_elem_range_n(100000, 100,  40000);
    imptree_single_elem_range_n(1000000, 100,  6000);

    btree_single_delete_n(100, 100,  30000);
    btree_single_delete_n(1000, 100,  5000);
    btree_single_delete_n(10000, 100, 1000);
    btree_single_delete_n(100000, 100,  80);
    btree_single_delete_n(1000000, 100, 30);


}



//---- unifying interfaces used in above benchmarks and its impls for 1) TeardownTree delete_range, 2) TeardownTree delete(), BTreeSet

mod bench_delete_range {
    use std::collections::BTreeSet;
    use std::ops::Range;
    use std::time;
    use std::iter::FromIterator;
    use std::fmt::{Formatter, Debug, Display, Result};
    use rand::{XorShiftRng, SeedableRng, Rng};
    use splay::SplaySet;

    use treap::TreapMap;
    use teardown_tree::{IntervalTeardownSet, IntervalTeardownMap, KeyInterval, Interval, Refill, TeardownSet, TeardownMap, ItemFilter, NoopFilter};
    use teardown_tree::util::make_teardown_seq;
    use teardown_tree::sink::{UncheckedVecRefSink};
    use super::{nanos, black_box};
    use super::ts::{Timestamp, new_timestamp, next_elapsed};

    pub type Tree = TeardownSet<usize>;
    pub type TreeBulk = TeardownTreeBulk;


    pub fn btree_single_delete_n(n: usize, rm_items: usize, iters: u64) {
        let mut rng = XorShiftRng::from_seed([1,2,3,4]);
        let mut elapsed_cycles = 0;

        let start = time::Instant::now();
        for _ in 0..iters {
            let mut btset = BTreeSet::new();
            for i in 0..n {
                btset.insert(i);
            }

            let keys = {
                let mut keys = vec![];
                let mut pool: Vec<_> = (0..n).collect();

                for i in 0..rm_items {
                    let n = rng.gen_range(0, n - i);
                    let next = pool.swap_remove(n);
                    keys.push(next);
                }

                keys
            };

            let mut ts: Timestamp = new_timestamp();
            for i in 0..rm_items {
                let x = btset.remove(&keys[i]);
                black_box(x);
            }
            elapsed_cycles += next_elapsed(&mut ts);
        }
        let elapsed = start.elapsed();
        let elapsed_nanos = nanos(elapsed);

        let avg_cycles = elapsed_cycles/iters;
        println!("average time to delete {} random elements from BTreeMap using remove(), {} elements: {}cy, total: {}ms", rm_items, n, avg_cycles, elapsed_nanos/1000000)
    }

    pub fn imptree_single_elem_range_n(n: usize, rm_items: usize, iters: u64) {
        let mut rng = XorShiftRng::from_seed([1,2,3,4]);
        let mut elapsed_cycles = 0;

        let elems: Vec<_> = (1..n+1).collect();

        let tree = TeardownTreeBulk(Tree::new(elems));
        let mut copy = tree.clone();
        let mut output = Vec::with_capacity(tree.0.size());

        let start = time::Instant::now();
        for _ in 0..iters {
            let keys = {
                let mut pool: Vec<_> = (1..n+1).collect();
                let mut keys = vec![];

                for i in 0..rm_items {
                    let r = rng.gen_range(0, n-i);
                    let next = pool.swap_remove(r);
                    keys.push(next);
                }

                keys
            };

            copy.refill(&tree);


            let mut ts: Timestamp = new_timestamp();
            for i in 0..rm_items {
                output.truncate(0);
                let x = copy.0.delete_range(keys[i]..keys[i]+1, UncheckedVecRefSink::new(&mut output));
                black_box(x);
            }
            elapsed_cycles += next_elapsed(&mut ts);
        }
        let elapsed = start.elapsed();
        let elapsed_nanos = nanos(elapsed);

        let avg_cycles = elapsed_cycles/iters;

        println!("average time to delete {} random elements from TeardownTree using delete_range(), {} elements: {}cy, total: {}ms", rm_items, n, avg_cycles, elapsed_nanos/1000000)
    }

    #[inline(never)]
    pub fn bench_refill<M: DataMaster>(n: usize, iters: u64) -> u64 {
        let elems: Vec<_> = (0..n).collect();
        let tree = build::<M>(elems);
        let mut copy = tree.cpy();
        let mut elapsed_cycles = 0;

        let start = time::Instant::now();
        for _ in 0..iters {
            copy = black_box(copy);
            copy.clear();
            let mut ts: Timestamp = new_timestamp();
            copy.refill(&tree);
            elapsed_cycles += next_elapsed(&mut ts);
        }
        let total = nanos(start.elapsed());

        let avg_cycles = elapsed_cycles/iters;
        println!("average time to refill {} with {} elements: {}cy, total: {}ms", M::descr_refill(), n, avg_cycles, total/1000000);
        avg_cycles
    }

    #[inline(never)]
    pub fn bench_refill_teardown_cycle<M: DataMaster>(n: usize, rm_items: usize, iters: u64) -> u64 {
        let mut rng = XorShiftRng::from_seed([1,2,3,4]);
        let elems: Vec<_> = (0..n).collect();

        let ranges = make_teardown_seq(n, rm_items, &mut rng);

        let tree = build::<M>(elems);
        let mut copy = tree.cpy();
        let mut output = Vec::with_capacity(tree.size());
        copy.delete_range(0..n, &mut output);
        output.truncate(0);

        let start = time::Instant::now();
        let mut ts: Timestamp = new_timestamp();
        for iter in 0..iters {
            copy.refill(&tree);
            for i in 0..ranges.len() {
                output.truncate(0);
                copy.delete_range(ranges[i].clone(), &mut output);
                output = black_box(output);
                let expected_len = ranges[i].end - ranges[i].start;
                assert!(output.len() == expected_len, "range={:?}, expected: {}, len: {}, iter={}, i={}, output={:?}, copy={:?}, {}", ranges[i], expected_len, output.len(), iter, i, output, &copy, &copy);
            }
            assert!(copy.size() == 0);
        }
        let elapsed_cycles = next_elapsed(&mut ts);
        let avg_cycles = elapsed_cycles/iters;
        let elapsed_nanos = nanos(start.elapsed());
        println!("average time to refill/tear down {}, {} elements in bulks of {} elements: {}cy, total: {}ms", M::descr_cycle(), n, rm_items, avg_cycles, elapsed_nanos/1000000);
        avg_cycles
    }


    fn build<M: DataMaster>(mut elems: Vec<usize>) -> M {
        let mut rng = XorShiftRng::from_seed([42,142,1,7832]);

        // shuffle the elements, so that the tree comes out balanced
        for i in 0..elems.len() {
            let pos = rng.gen_range(i, elems.len());

            let tmp = elems[pos];
            elems[pos] = elems[i];
            elems[i] = tmp;
        }

        M::build(elems)
    }


    pub trait DataMaster: Sized+Display {
        type Cpy: DataCopy<Master = Self>;

        fn build(elems: Vec<usize>) -> Self;
        fn cpy(&self) -> Self::Cpy;
        fn size(&self) -> usize;
        fn descr_cycle() -> String;
        fn descr_refill() -> String;
    }

    pub trait DataCopy: Display+Debug {
        type Master: DataMaster;
        type T: Debug;

        fn delete_range(&mut self, range: Range<usize>, output: &mut Vec<Self::T>);
        #[inline(never)] fn refill(&mut self, master: &Self::Master);
        fn size(&self) -> usize;
        fn clear(&mut self);
        fn as_vec(&self) -> Vec<Self::T>;
    }


    /// for benchmarking TeardownSet::delete_range()
    #[derive(Clone, Debug)]
    pub struct TeardownTreeBulk(TeardownSet<usize>);

    impl DataMaster for TeardownTreeBulk {
        type Cpy = TeardownTreeBulk;

        fn build(elems: Vec<usize>) -> Self {
            TeardownTreeBulk(TeardownSet::new(elems))
        }

        fn cpy(&self) -> Self {
            self.clone()
        }

        fn size(&self) -> usize {
            self.0.size()
        }

        fn descr_cycle() -> String {
            "TeardownSet::delete_range()".to_string()
        }

        fn descr_refill() -> String {
            "TeardownSet".to_string()
        }
    }

    impl DataCopy for TeardownTreeBulk {
        type Master = TeardownTreeBulk;
        type T = usize;

        fn delete_range(&mut self, range: Range<usize>, output: &mut Vec<usize>) {
            self.0.delete_range(range, UncheckedVecRefSink::new(output));
        }

        #[inline(never)]
        fn refill(&mut self, master: &Self::Master) {
            self.0.refill(&master.0)
        }

        fn size(&self) -> usize {
            self.0.size()
        }

        fn clear(&mut self) {
            self.0.clear();
        }

        fn as_vec(&self) -> Vec<Self::T> {
            self.0.iter().cloned().collect()
        }
    }

    impl Display for TeardownTreeBulk {
        fn fmt(&self, fmt: &mut Formatter) -> Result {
            Display::fmt(&self.0, fmt)
        }
    }


    /// for benchmarking TeardownSet::delete()
    #[derive(Clone, Debug)]
    pub struct TeardownTreeSingle(TeardownSet<usize>);

    impl DataMaster for TeardownTreeSingle {
        type Cpy = TeardownTreeSingle;

        fn build(elems: Vec<usize>) -> Self {
            TeardownTreeSingle(TeardownSet::new(elems))
        }

        fn cpy(&self) -> Self {
            self.clone()
        }

        fn size(&self) -> usize {
            self.0.size()
        }

        fn descr_cycle() -> String {
            "TeardownSet::delete()".to_string()
        }

        fn descr_refill() -> String {
            "TeardownSet".to_string()
        }
    }

    impl DataCopy for TeardownTreeSingle {
        type Master = TeardownTreeSingle;
        type T = usize;

        fn delete_range(&mut self, range: Range<usize>, output: &mut Vec<usize>) {
            for i in range {
                if self.0.delete(&i) {
                    output.push(i);
                }
            }
        }

        #[inline(never)]
        fn refill(&mut self, master: &Self::Master) {
            self.0.refill(&master.0)
        }

        fn size(&self) -> usize {
            self.0.size()
        }

        fn clear(&mut self) {
            self.0.clear();
        }

        fn as_vec(&self) -> Vec<Self::T> {
            self.0.iter().cloned().collect()
        }
    }

    impl Display for TeardownTreeSingle {
        fn fmt(&self, fmt: &mut Formatter) -> Result {
            Display::fmt(&self.0, fmt)
        }
    }


    #[derive(Debug)]
    pub struct BTreeSetMaster(BTreeSet<usize>);

    /// for benchmarking BTreeSet::remove()
    impl DataMaster for BTreeSetMaster {
        type Cpy = BTreeSetCopy;

        fn build(elems: Vec<usize>) -> Self {
            let mut set = BTreeSet::new();

            for elem in elems.into_iter() {
                set.insert(elem);
            }

            BTreeSetMaster(set)
        }

        fn cpy(&self) -> Self::Cpy {
            BTreeSetCopy { set: self.0.clone() }
        }

        fn size(&self) -> usize {
            self.0.len()
        }

        fn descr_cycle() -> String {
            "BTreeSet::remove()".to_string()
        }

        fn descr_refill() -> String {
            "BTreeSet".to_string()
        }
    }

    #[derive(Debug)]
    pub struct BTreeSetCopy {
        set: BTreeSet<usize>
    }

    impl DataCopy for BTreeSetCopy {
        type Master = BTreeSetMaster;
        type T = usize;

        fn delete_range(&mut self, range: Range<usize>, output: &mut Vec<usize>) {
            for i in range {
                if self.set.remove(&i) {
                    output.push(i);
                }
            }
        }

        #[inline(never)]
        fn refill(&mut self, master: &Self::Master) {
            assert!(self.set.is_empty(), "size={}", self.set.len());
            self.set = master.0.clone();
        }

        fn size(&self) -> usize {
            self.set.len()
        }

        fn clear(&mut self) {
            self.set.clear();
        }

        fn as_vec(&self) -> Vec<Self::T> {
            self.set.iter().cloned().collect()
        }
    }

    impl Display for BTreeSetMaster {
        fn fmt(&self, fmt: &mut Formatter) -> Result {
            Debug::fmt(&self.0, fmt)
        }
    }

    impl Display for BTreeSetCopy {
        fn fmt(&self, fmt: &mut Formatter) -> Result {
            Debug::fmt(&self.set, fmt)
        }
    }


    //---- benchmarking Treap split/join ---------------------------------------------------------------
    pub struct TreapMaster(TreapMap<usize, ()>);

    pub struct TreapCopy(TreapMap<usize, ()>);

    impl DataMaster for TreapMaster {
        type Cpy = TreapCopy;

        fn build(elems: Vec<usize>) -> Self {
            let iter = elems.into_iter().map(|x| (x, ()));
            TreapMaster(TreapMap::from_iter(iter))
        }

        fn cpy(&self) -> Self::Cpy {
            TreapCopy(self.0.clone())
        }

        fn size(&self) -> usize {
            self.0.len()
        }

        fn descr_cycle() -> String {
            "Treap::delete_range()".to_string()
        }

        fn descr_refill() -> String {
            "Treap".to_string()
        }
    }

    impl DataCopy for TreapCopy {
        type Master = TreapMaster;
        type T = usize;

        fn delete_range(&mut self, range: Range<usize>, output: &mut Vec<usize>) {
            self.0.remove_range(range, output);
        }

        #[inline(never)]
        fn refill(&mut self, master: &Self::Master) {
            self.0 = master.0.clone()
        }

        fn size(&self) -> usize {
            self.0.len()
        }

        fn clear(&mut self) {
            self.0.clear();
        }

        fn as_vec(&self) -> Vec<Self::T> {
            self.0.iter_ordered().map(|(&x, _)| x).collect()
        }
    }

    impl Display for TreapMaster {
        fn fmt(&self, _: &mut Formatter) -> Result {
            unimplemented!()
        }
    }

    impl Display for TreapCopy {
        fn fmt(&self, _: &mut Formatter) -> Result {
            unimplemented!()
        }
    }

    impl Debug for TreapCopy {
        fn fmt(&self, _: &mut Formatter) -> Result {
            unimplemented!()
        }
    }


    //---- benchmarking SplayTree split/join ---------------------------------------------------------------
    pub struct SplayMaster(SplaySet<usize>);

    pub struct SplayCopy(SplaySet<usize>);

    impl DataMaster for SplayMaster {
        type Cpy = SplayCopy;

        fn build(elems: Vec<usize>) -> Self {
            SplayMaster(SplaySet::from_iter(elems.into_iter()))
        }

        fn cpy(&self) -> Self::Cpy {
            SplayCopy(self.0.clone())
        }

        fn size(&self) -> usize {
            self.0.len()
        }

        fn descr_cycle() -> String {
            "SplayTree::remove_range()".to_string()
        }

        fn descr_refill() -> String {
            "SplayTree".to_string()
        }
    }

    impl DataCopy for SplayCopy {
        type Master = SplayMaster;
        type T = usize;

        fn delete_range(&mut self, range: Range<usize>, output: &mut Vec<usize>) {
            self.0.remove_range(&range.start .. &range.end, output);
        }

        #[inline(never)]
        fn refill(&mut self, master: &Self::Master) {
            self.0 = master.0.clone()
        }

        fn size(&self) -> usize {
            self.0.len()
        }

        fn clear(&mut self) {
            self.0.clear();
        }

        fn as_vec(&self) -> Vec<Self::T> {
            self.0.clone().into_iter().collect()
        }
    }

    impl Display for SplayMaster {
        fn fmt(&self, _: &mut Formatter) -> Result {
            unimplemented!()
        }
    }

    impl Display for SplayCopy {
        fn fmt(&self, _: &mut Formatter) -> Result {
            unimplemented!()
        }
    }

    impl Debug for SplayCopy {
        fn fmt(&self, _: &mut Formatter) -> Result {
            unimplemented!()
        }
    }


    /// for benchmarking IntervalTeardownSet::delete_range()
    #[derive(Clone, Debug)]
    pub struct IntervalTreeBulk(IntervalTeardownSet<KeyInterval<usize>>);

    impl DataMaster for IntervalTreeBulk {
        type Cpy = IntervalTreeBulk;

        fn build(elems: Vec<usize>) -> Self {
            let elems = elems.into_iter().map(|x| KeyInterval::new(x, x)).collect();
            IntervalTreeBulk(IntervalTeardownSet::new(elems))
        }

        fn cpy(&self) -> Self {
            self.clone()
        }

        fn size(&self) -> usize {
            self.0.size()
        }

        fn descr_cycle() -> String {
            "IntervalTeardownSet::delete_range()".to_string()
        }

        fn descr_refill() -> String {
            "IntervalTeardownSet".to_string()
        }
    }

    impl DataCopy for IntervalTreeBulk {
        type Master = IntervalTreeBulk;
        type T = KeyInterval<usize>;

        fn delete_range(&mut self, range: Range<usize>, output: &mut Vec<Self::T>) {
            self.0.delete_overlap(&KeyInterval::new(range.start, range.end), UncheckedVecRefSink::new(output));
        }

        #[inline(never)]
        fn refill(&mut self, master: &Self::Master) {
            self.0.refill(&master.0)
        }

        fn size(&self) -> usize {
            self.0.size()
        }

        fn clear(&mut self) {
            self.0.clear();
        }

        fn as_vec(&self) -> Vec<Self::T> {
            self.0.iter().cloned().collect()
        }
    }


    impl Display for IntervalTreeBulk {
        fn fmt(&self, fmt: &mut Formatter) -> Result {
            Display::fmt(&self.0, fmt)
        }
    }

    /// for benchmarking IntervalTeardownSet::filter_range()
    #[derive(Clone, Debug)]
    pub struct FilteredIntervalTreeBulk(IntervalTeardownSet<KeyInterval<usize>>);

    impl DataMaster for FilteredIntervalTreeBulk {
        type Cpy = FilteredIntervalTreeBulk;

        fn build(elems: Vec<usize>) -> Self {
            let elems = elems.into_iter().map(|x| KeyInterval::new(x, x)).collect();
            FilteredIntervalTreeBulk(IntervalTeardownSet::new(elems))
        }

        fn cpy(&self) -> Self {
            self.clone()
        }

        fn size(&self) -> usize {
            self.0.size()
        }

        fn descr_cycle() -> String {
            "IntervalTeardownSet::filter_range()".to_string()
        }

        fn descr_refill() -> String {
            "IntervalTeardownSet".to_string()
        }
    }

    impl DataCopy for FilteredIntervalTreeBulk {
        type Master = FilteredIntervalTreeBulk;
        type T = KeyInterval<usize>;

        fn delete_range(&mut self, range: Range<usize>, output: &mut Vec<Self::T>) {
            self.0.filter_overlap(&KeyInterval::new(range.start, range.end), NoopFilter, UncheckedVecRefSink::new(output));
        }

        #[inline(never)]
        fn refill(&mut self, master: &Self::Master) {
            self.0.refill(&master.0)
        }

        fn size(&self) -> usize {
            self.0.size()
        }

        fn clear(&mut self) {
            self.0.clear();
        }

        fn as_vec(&self) -> Vec<Self::T> {
            self.0.iter().cloned().collect()
        }
    }


    impl Display for FilteredIntervalTreeBulk {
        fn fmt(&self, fmt: &mut Formatter) -> Result {
            Display::fmt(&self.0, fmt)
        }
    }



    /// for benchmarking TeardownSet::filter_range()
    #[derive(Clone, Debug)]
    pub struct TeardownSetFilter(TeardownSet<usize>);

    impl DataMaster for TeardownSetFilter {
        type Cpy = TeardownSetFilter;

        fn build(elems: Vec<usize>) -> Self {
            TeardownSetFilter(TeardownSet::new(elems))
        }

        fn cpy(&self) -> Self {
            self.clone()
        }

        fn size(&self) -> usize {
            self.0.size()
        }

        fn descr_cycle() -> String {
            "TeardownSet::filter_range()".to_string()
        }

        fn descr_refill() -> String {
            "TeardownSet".to_string()
        }
    }

    impl DataCopy for TeardownSetFilter {
        type Master = TeardownSetFilter;
        type T = usize;

        fn delete_range(&mut self, range: Range<usize>, output: &mut Vec<usize>) {
            self.0.filter_range(range, AcceptingFilter, UncheckedVecRefSink::new(output));
        }

        #[inline(never)]
        fn refill(&mut self, master: &Self::Master) {
            self.0.refill(&master.0)
        }

        fn size(&self) -> usize {
            self.0.size()
        }

        fn clear(&mut self) {
            self.0.clear();
        }

        fn as_vec(&self) -> Vec<Self::T> {
            self.0.iter().cloned().collect()
        }
    }

    impl Display for TeardownSetFilter {
        fn fmt(&self, fmt: &mut Formatter) -> Result {
            Display::fmt(&self.0, fmt)
        }
    }



    /// for benchmarking IntervalTeardownSet::filter_overlap()
    #[derive(Clone, Debug)]
    pub struct IntervalSetFilter(IntervalTeardownSet<usize>);

    impl DataMaster for IntervalSetFilter {
        type Cpy = IntervalSetFilter;

        fn build(elems: Vec<usize>) -> Self {
            IntervalSetFilter(IntervalTeardownSet::new(elems))
        }

        fn cpy(&self) -> Self {
            self.clone()
        }

        fn size(&self) -> usize {
            self.0.size()
        }

        fn descr_cycle() -> String {
            "IntervalTeardownSet::filter_overlap()".to_string()
        }

        fn descr_refill() -> String {
            "IntervalTeardownSet".to_string()
        }
    }

    impl DataCopy for IntervalSetFilter {
        type Master = IntervalSetFilter;
        type T = usize;

        fn delete_range(&mut self, range: Range<usize>, output: &mut Vec<usize>) {
            self.0.filter_overlap(&KeyInterval::new(range.start, range.end), AcceptingFilter, UncheckedVecRefSink::new(output));
        }

        #[inline(never)]
        fn refill(&mut self, master: &Self::Master) {
            self.0.refill(&master.0)
        }

        fn size(&self) -> usize {
            self.0.size()
        }

        fn clear(&mut self) {
            self.0.clear();
        }

        fn as_vec(&self) -> Vec<Self::T> {
            self.0.iter().cloned().collect()
        }
    }

    impl Display for IntervalSetFilter {
        fn fmt(&self, fmt: &mut Formatter) -> Result {
            Display::fmt(&self.0, fmt)
        }
    }




    #[derive(Clone, Debug)]
    pub struct AcceptingFilter;

    impl<K: Ord+Clone> ItemFilter<K> for AcceptingFilter {
        #[inline(always)] fn accept(&mut self, k: &K) -> bool {
            black_box(k);
            black_box(true)
        }

        #[inline(always)] fn is_noop() -> bool { false }
    }
}



mod ts {
    use super::black_box;
    use x86::bits64::time::rdtsc;

    pub type Timestamp = u64;

    #[inline]
    pub fn new_timestamp() -> Timestamp {
        // we cannot use rdtscp, it's bugged (some kind of memory or register corruption)

        // TODO: check whether a fence is really needed here. it sure is very expensive
//        unsafe { black_box(rdtsc()) }
        unsafe { rdtsc() }
    }

    #[inline]
    pub fn next_elapsed(prev_timestamp: &mut Timestamp) -> u64 {
        let timestamp = new_timestamp();
        let elapsed = timestamp - *prev_timestamp;
        *prev_timestamp = timestamp;
        elapsed
    }
}


pub fn black_box<T>(dummy: T) -> T {
    use std::ptr;
    use std::mem::forget;

    unsafe {
        let ret = ptr::read_volatile(&dummy as *const T);
        forget(dummy);
        ret
    }
}

#[inline]
fn nanos(d: Duration) -> u64 {
    d.as_secs()*1000000000 + d.subsec_nanos() as u64
}
