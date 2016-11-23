#![feature(test)]
extern crate test;
extern crate rand;

mod base;
mod delete_bulk;







#[cfg(test)]
mod tests {
    use base::{Item, ImplicitTree, Node};
    use delete_bulk::{TraversalDecision, TraversalDriver};

    type Tree = ImplicitTree<usize>;

    impl Item for usize {
        type Key = usize;

        fn ord(&self) -> Self::Key {
            *self
        }
    }


    struct DriverFromTo {
        from: usize,
        to: usize
    }

    impl DriverFromTo {
        pub fn new(from: usize, to: usize) -> DriverFromTo {
            DriverFromTo { from:from, to:to }
        }
    }

    impl TraversalDriver<usize> for DriverFromTo {
        fn decide(&mut self, node: &mut Node<usize>) -> TraversalDecision {
            let x = node.item.unwrap();
            let left = self.from <= x;
            let right = x <= self.to;
            let consume = left && right;

            TraversalDecision { traverse_left: left, traverse_right: right, consume_curr: consume }
        }
    }


    #[test]
    fn build() {
        Tree::new(vec![1]);
        Tree::new(vec![1, 2]);
        Tree::new(vec![1, 2, 3]);
        Tree::new(vec![1, 2, 3, 4]);
        Tree::new(vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn delete_bulk4() {
        let mut tree = Tree::new(vec![1, 2, 3, 4]);
        let mut drv = DriverFromTo::new(2,2);
        let mut output = Vec::with_capacity(tree.size());
        tree.delete_bulk(&mut drv, &mut output);
        println!("tree: {:?}", &tree);
        println!("output: {:?}", &output);
//        panic!();
    }


    fn test_prebuilt(items: &[usize], from_to: (usize, usize),
                     expect_tree: &[(usize, usize)], expect_out: &[usize]) {

        let nodes: Vec<Node<usize>> = mk_prebuilt(items);
        let mut tree = Tree::with_nodes(nodes);
        let (from, to) = from_to;
        let mut drv = DriverFromTo::new(from, to);
        let mut output = Vec::with_capacity(tree.size());
        tree.delete_bulk(&mut drv, &mut output);
        assert_eq!(format!("{:?}", &tree), format!("{:?}", expect_tree));
        assert_eq!(format!("{:?}", &output), format!("{:?}", expect_out));
    }


    #[test]
    fn delete_bulk_prebuilt() {
        test_prebuilt(&[1, 0, 2, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 4], (1,1),
                      &[(2, 3), (0, 0), (3, 2), (0, 0), (0, 0), (0, 0), (4, 1)], &[1]);

        test_prebuilt(&[1, 0, 2, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 4], (2,2),
                      &[(1, 3), (0, 0), (3, 2), (0, 0), (0, 0), (0, 0), (4, 1)], &[2]);

        test_prebuilt(&[1, 0, 2, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 4], (3,3),
                      &[(1, 3), (0, 0), (2, 2), (0, 0), (0, 0), (0, 0), (4, 1)], &[3]);

        test_prebuilt(&[1, 0, 2, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 4], (4,4),
                      &[(1, 3), (0, 0), (2, 2), (0, 0), (0, 0), (0, 0), (3, 1)], &[4]);


        test_prebuilt(&[4, 3, 0, 2, 0, 0, 0, 1], (1,1),
                      &[(4, 3), (3, 2), (0, 0), (2, 1)], &[1]);

        test_prebuilt(&[4, 3, 0, 2, 0, 0, 0, 1], (2,2),
                      &[(4, 3), (3, 2), (0, 0), (1, 1)], &[2]);

        test_prebuilt(&[4, 3, 0, 2, 0, 0, 0, 1], (3,3),
                      &[(4, 3), (2, 2), (0, 0), (1, 1)], &[3]);

        test_prebuilt(&[4, 3, 0, 2, 0, 0, 0, 1], (4,4),
                      &[(3, 3), (2, 2), (0, 0), (1, 1)], &[4]);

        test_prebuilt(&[1, 0, 3, 0, 0, 2, 4], (1,2),
                      &[(3, 2), (0, 0), (4, 1)], &[1, 2]);


        test_prebuilt(&[3, 1, 4, 0, 2], (2,4),
                      &[(1, 1)], &[3,2,4]);

        test_prebuilt(&[4, 2, 0, 1, 3], (3,4),
                      &[(2, 2), (1,1)], &[4,3]);
    }


    fn mk_prebuilt(items: &[usize]) -> Vec<Node<usize>> {
        let mut nodes: Vec<_> = items.iter().map(|&x| if x==0 {
            Node { item:None, height: 0 }
        } else {
            Node { item: Some(x), height: 1}
        }).collect();

        for i in (1..items.len()).rev() {
            if nodes[i].height != 0 {
                let pari = Tree::parenti(i);
                nodes[pari].height = ::std::cmp::max(nodes[pari].height, 1 + nodes[i].height);
            }
        }

        nodes
    }




    use std::ops::Range;

    #[derive(Debug)]
    struct TreeRangeInfo {
        range: Range<usize>,
        root_idx: usize
    }


    #[test]
    fn delete_bulk_exhaustive() {
        for i in 1..7 {
            delete_bulk_exhaustive_n(i);
        }
    }

    fn delete_bulk_exhaustive_n(n: usize) {
        let elems: Vec<_> = (1..n+1).collect();
        println!("exhaustive {}: elems={:?} ------------------------", n, &elems);

        let mut stack = vec![TreeRangeInfo { range: (1..n+1), root_idx: 0 }];
        let mut items: Vec<usize> = vec![0; 1 << n];
        delete_bulk_exhaustive_rec(&mut stack, &mut items);
    }

    fn delete_bulk_exhaustive_rec(stack: &mut Vec<TreeRangeInfo>, items: &mut Vec<usize>) {
        if stack.is_empty() {
            let nodes: Vec<Node<usize>> = mk_prebuilt(items);
            let tree = Tree::with_nodes(nodes);
            delete_bulk_exhaustive_with_tree(tree);
        } else {
            let info = stack.pop().unwrap();
            let (lefti, righti) = (Tree::lefti(info.root_idx), Tree::righti(info.root_idx));
            for i in info.range.clone() {
                items[info.root_idx] = i;

                let mut pushed = 0;
                if info.range.start < i {
                    let range1 = info.range.start .. i;
                    stack.push(TreeRangeInfo { range: range1, root_idx: lefti });
                    pushed += 1;
                }

                if i+1 < info.range.end {
                    let range2 = i+1 .. info.range.end;
                    stack.push(TreeRangeInfo { range: range2, root_idx: righti });
                    pushed += 1;
                }

                delete_bulk_exhaustive_rec(stack, items);

                for _ in 0..pushed {
                    stack.pop();
                }
            }

            items[info.root_idx] = 0;
            stack.push(info);
        }
    }


    fn delete_bulk_exhaustive_with_tree(tree: Tree) {
        let n = tree.size();
        let mut output = Vec::with_capacity(n);
        for i in 1..n+1 {
            for j in i..n+1 {
                let mut tree_mod = tree.clone();
                let mut drv = DriverFromTo::new(i, j);
//                println!("from={}, to={}", i, j);
                output.truncate(0);
                tree_mod.delete_bulk(&mut drv, &mut output);
                delete_bulk_exhaustive_check(n, i, j, &mut output, tree_mod, &tree);
            }
        }
    }

    fn delete_bulk_exhaustive_check(n: usize, i: usize, j: usize, output: &mut Vec<usize>, tree_mod: Tree, tree_orig: &Tree) {
        assert!(output.len() == j-i+1);
        assert!(tree_mod.size() + output.len() == n);

        output.sort();
        assert_eq!(output, &(i..j+1).collect::<Vec<_>>());
        check_bst(&tree_mod, &output, tree_orig, 0);
    }

    fn check_bst(tree: &Tree, output: &Vec<usize>, tree_orig: &Tree, idx: usize) -> Option<(usize, usize)> {
        if tree.size() == 0 {
            return None;
        }

        let node = tree.node(idx);
        if node.height == 0 || node.item.is_none() {
            assert!(node.height == 0 && node.item.is_none());
            return None;
        } else {
            let item = node.item.unwrap();
            let left = check_bst(tree, output, tree_orig, Tree::lefti(idx));
            let right = check_bst(tree, output, tree_orig, Tree::righti(idx));

            let min =
                if let Some((lmin, lmax)) = left {
                    assert!(lmax < item, "tree_orig: {:?}, tree: {:?}, output: {:?}", tree_orig, tree, output);
                    lmin
                } else {
                    item
                };
            let max =
                if let Some((rmin, rmax)) = right {
                    assert!(item < rmin, "tree_orig: {:?}, tree: {:?}, output: {:?}", tree_orig, tree, output);
                    rmax
                } else {
                    item
                };

            return Some((min, max));
        }
    }


    //---- benchmarks ------------------------------------------------------------------------------
    use test::Bencher;
    use test;

    #[bench]
    fn bench_bulk_delete_00100(bencher: &mut Bencher) {
        bench_bulk_delete_n(100, bencher);
    }

    #[bench]
    fn bench_bulk_delete_01022(bencher: &mut Bencher) {
        bench_bulk_delete_n(1022, bencher);
    }

    #[bench]
    fn bench_bulk_delete_01023(bencher: &mut Bencher) {
        bench_bulk_delete_n(1023, bencher);
    }

    #[bench]
    fn bench_bulk_delete_02046(bencher: &mut Bencher) {
        bench_bulk_delete_n(2046, bencher);
    }

    #[bench]
    fn bench_bulk_delete_02047(bencher: &mut Bencher) {
        bench_bulk_delete_n(2047, bencher);
    }

    #[bench]
    fn bench_bulk_delete_04094(bencher: &mut Bencher) {
        bench_bulk_delete_n(4094, bencher);
    }

    #[bench]
    fn bench_bulk_delete_04095(bencher: &mut Bencher) {
        bench_bulk_delete_n(4095, bencher);
    }

    #[bench]
    fn bench_bulk_delete_05000(bencher: &mut Bencher) {
        bench_bulk_delete_n(5000, bencher);
    }

    #[bench]
    fn bench_bulk_delete_08190(bencher: &mut Bencher) {
        bench_bulk_delete_n(8190, bencher);
    }

    #[bench]
    fn bench_bulk_delete_08191(bencher: &mut Bencher) {
        bench_bulk_delete_n(8191, bencher);
    }

    #[bench]
    fn bench_bulk_delete_10000(bencher: &mut Bencher) {
        bench_bulk_delete_n(10000, bencher);
    }

    #[bench]
    fn bench_bulk_delete_16000(bencher: &mut Bencher) {
        bench_bulk_delete_n(16000, bencher);
    }

    #[bench]
    fn bench_bulk_delete_16381(bencher: &mut Bencher) {
        bench_bulk_delete_n(16381, bencher);
    }

    #[bench]
    fn bench_bulk_delete_16382(bencher: &mut Bencher) {
        bench_bulk_delete_n(test::black_box(16382), bencher);
    }

    #[bench]
    fn bench_bulk_delete_16383(bencher: &mut Bencher) {
        bench_bulk_delete_n(test::black_box(16383), bencher);
    }

    #[bench]
    fn bench_bulk_delete_25000(bencher: &mut Bencher) {
        bench_bulk_delete_n(25000, bencher);
    }

    #[bench]
    fn bench_bulk_delete_50000(bencher: &mut Bencher) {
        bench_bulk_delete_n(50000, bencher);
    }

//    #[bench]
//    fn bench_bulk_delete_100000(bencher: &mut Bencher) {
//        bench_bulk_delete_n(100000, bencher);
//    }
//
//    #[bench]
//    fn bench_bulk_delete_10000000(bencher: &mut Bencher) {
//        bench_bulk_delete_n(10000000, bencher);
//    }

    #[inline(never)]
    fn bench_bulk_delete_n(n: usize, bencher: &mut Bencher) {
        let elems: Vec<_> = (1..n+1).collect();

        let perm = {
            // generate a random permutation
            let mut pool: Vec<_> = (1..101).collect();
            let mut perm = vec![];

            use rand::{XorShiftRng, SeedableRng, Rng};

            let mut rng = XorShiftRng::from_seed([1,2,3,4]);

            for i in 0..100 {
                let n: u32 = rng.gen_range(0, 100-i);
                let next = pool.swap_remove(n as usize);
                perm.push(next);
            }

            perm
        };


        let mut tree = Tree::new(elems);
        let mut copy = tree.clone();
        let mut output = Vec::with_capacity(tree.size());

        bencher.iter(|| {
            copy.refill(&tree);
            for i in 0..100 {
                output.truncate(0);
                let x = perm[i];
                copy.delete_bulk(&mut DriverFromTo::new((x-1)*n/100, x*n/100), &mut output);
                test::black_box(output.len());
            }
        });
    }
}
