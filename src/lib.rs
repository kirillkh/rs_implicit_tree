//#![feature(specialization)]
//#![feature(unique)]
#![cfg_attr(feature = "unstable", feature(test))]

//#![cfg_attr(test, feature(plugin))]
//#![cfg_attr(test, plugin(quickcheck_macros))]
#[cfg(test)] #[macro_use] extern crate quickcheck;


extern crate rand;

mod base;
mod applied;
mod external_api;

mod rust_bench;

pub use self::external_api::{IntervalTeardownTree, PlainTeardownTree, TeardownTreeRefill};



#[cfg(test)]
mod plain_tests {
    use base::{TreeBase, TreeWrapper, Node, lefti, righti, parenti};
    use base::validation::{check_bst, check_integrity};
    use applied::plain_tree::PlainDeleteInternal;
    use external_api::{PlainTeardownTree, PlainTreeWrapperAccess};

    type Tree = TreeWrapper<usize>;


    #[test]
    fn build() {
        Tree::new(vec![1]);
        Tree::new(vec![1, 2]);
        Tree::new(vec![1, 2, 3]);
        Tree::new(vec![1, 2, 3, 4]);
        Tree::new(vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn delete_range1() {
        delete_range_n(1);
    }

    #[test]
    fn delete_range2() {
        delete_range_n(1);
    }

    #[test]
    fn delete_range3() {
        delete_range_n(1);
    }

    #[test]
    fn delete_range4() {
        delete_range_n(4);
    }


    fn delete_range_n(n: usize) {
        let tree = Tree::new((1..n+1).collect::<Vec<_>>());
        delete_range_exhaustive_with_tree(tree);
    }


    fn test_prebuilt(items: &[usize], from_to: (usize, usize),
                     expect_out: &[usize]) {

        let nodes: Vec<Option<Node<usize>>> = mk_prebuilt(items);
        let mut tree = Tree::with_nodes(nodes);
        let (from, to) = from_to;

        let mut output = Vec::with_capacity(tree.size());

        let mut expect = expect_out.to_vec();
        expect.sort();

//        println!("tree={:?}, range=({}, {}), {}", &tree, from, to, &tree);
        let tree_orig = tree.clone();
        tree.delete_range(from, to, &mut output);

        let tree_mod = tree.clone();
        delete_range_check(items.iter().filter(|&&x| x!=0).count(), from, to, &mut output, tree_mod, &tree_orig);

        assert_eq!(format!("{:?}", &output), format!("{:?}", expect));
    }

    #[test]
    fn delete_range_prebuilt() {
        test_prebuilt(&[1], (1,1),
                      &[1]);

        test_prebuilt(&[1, 0, 2], (1,1),
                      &[1]);

        test_prebuilt(&[1, 0, 2], (2,2),
                      &[2]);

        test_prebuilt(&[3, 2, 0, 1], (1,3),
                      &[3,2,1]);

        test_prebuilt(&[3, 2, 4, 1], (1,3),
                      &[3,2,1]);

        test_prebuilt(&[3, 1, 4, 0, 2], (2,4),
                      &[3,4,2]);

        test_prebuilt(&[4, 2, 0, 1, 3], (3,4),
                      &[4,3]);


        test_prebuilt(&[4, 3, 0, 2, 0, 0, 0, 1], (1,1),
                      &[1]);

        test_prebuilt(&[4, 3, 0, 2, 0, 0, 0, 1], (2,2),
                      &[2]);

        test_prebuilt(&[4, 3, 0, 2, 0, 0, 0, 1], (3,3),
                      &[3]);

        test_prebuilt(&[4, 3, 0, 2, 0, 0, 0, 1], (4,4),
                      &[4]);

        test_prebuilt(&[1, 0, 3, 0, 0, 2, 4], (1,2),
                      &[1, 2]);


        test_prebuilt(&[1, 0, 2, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 4], (1,1),
                      &[1]);

        test_prebuilt(&[1, 0, 2, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 4], (2,2),
                      &[2]);

        test_prebuilt(&[1, 0, 2, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 4], (3,3),
                      &[3]);

        test_prebuilt(&[1, 0, 2, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 4], (4,4),
                      &[4]);

        test_prebuilt(&[1, 0, 4, 0, 0, 2, 0, 0, 0, 0, 0, 0, 3], (1,4),
                      &[1,2,3,4]);

        test_prebuilt(&[6, 4, 0, 1, 5, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3], (4,6),
                      &[6,4,5]);

        test_prebuilt(&[1, 0, 2, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 3], (1,1),
                      &[1]);

        test_prebuilt(&[1, 0, 2, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4], (1,2),
                      &[1,2]);
    }


    fn mk_prebuilt(items: &[usize]) -> Vec<Option<Node<usize>>> {
        let nodes: Vec<_> = items.iter().map(|&x| if x==0 {
            None
        } else {
            Some(Node::new(x))
        }).collect();

        nodes
    }




    use std::ops::Range;

    #[derive(Debug)]
    struct TreeRangeInfo {
        range: Range<usize>,
        root_idx: usize
    }


    #[test]
    fn delete_range_exhaustive() {
        for i in 1..8 {
            delete_range_exhaustive_n(i);
        }
    }

    #[test]
    fn delete_single_exhaustive() {
        for i in 1..8 {
            delete_single_exhaustive_n(i);
        }
    }

    fn delete_single_exhaustive_n(n: usize) {
        test_exhaustive_n(n, &|tree| delete_single_exhaustive_with_tree(tree));
    }

    fn delete_range_exhaustive_n(n: usize) {
        test_exhaustive_n(n, &|tree| delete_range_exhaustive_with_tree(tree));
    }

    fn test_exhaustive_n<F>(n: usize, check: &F)
                        where F: Fn(Tree) -> () {
        let elems: Vec<_> = (1..n+1).collect();
        println!("exhaustive {}: elems={:?} ------------------------", n, &elems);

        let mut stack = vec![TreeRangeInfo { range: (1..n+1), root_idx: 0 }];
        let mut items: Vec<usize> = vec![0; 1 << n];
        test_exhaustive_rec(&mut stack, &mut items, check);
    }

    fn test_exhaustive_rec<F>(stack: &mut Vec<TreeRangeInfo>, items: &mut Vec<usize>, check: &F)
                                                            where F: Fn(Tree) -> () {
        if stack.is_empty() {
            let nodes: Vec<Option<Node<usize>>> = mk_prebuilt(items);
            let tree = Tree::with_nodes(nodes);
            check(tree);
        } else {
            let info = stack.pop().unwrap();
            let (lefti, righti) = (lefti(info.root_idx), righti(info.root_idx));
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

                test_exhaustive_rec(stack, items, check);

                for _ in 0..pushed {
                    stack.pop();
                }
            }

            items[info.root_idx] = 0;
            stack.push(info);
        }
    }


    fn delete_single_exhaustive_with_tree(tree: Tree) {
        let n = tree.size();
        let mut output = Vec::with_capacity(n);
        for i in 1..n+1 {
            output.truncate(0);
            let mut tree_mod = tree.clone();
//                println!("tree={:?}, from={}, to={}", &tree, i, j);
            let deleted = tree_mod.delete(&i);
            debug_assert!(deleted.is_some());
            output.push(i);
            delete_range_check(n, i, i, &mut output, tree_mod, &tree);
        }
    }

    fn delete_range_exhaustive_with_tree(tree: Tree) {
        let n = tree.size();
        let mut output = Vec::with_capacity(n);
        for i in 1..n+1 {
            for j in i..n+1 {
                let mut tree_mod = tree.clone();
//                println!("tree={:?}, from={}, to={}, {}", &tree, i, j, &tree);
                output.truncate(0);
                tree_mod.delete_range(i, j, &mut output);
                delete_range_check(n, i, j, &mut output, tree_mod, &tree);
            }
        }
    }

    fn delete_range_check(n: usize, from: usize, to: usize, output: &mut Vec<usize>, tree_mod: Tree, tree_orig: &Tree) {
        debug_assert!(output.len() == to-from+1, "tree'={:?}, tree={}, tree_mod={}, interval=({}, {}), expected output len={}, got: {:?}", tree_orig, tree_orig, tree_mod, from, to, to-from+1, output);
        debug_assert!(tree_mod.size() + output.len() == n, "tree'={:?}, tree={}, tree_mod={}, sz={}, output={:?}, n={}", tree_orig, tree_orig, tree_mod, tree_mod.size(), output, n);

//        output.sort();
        assert_eq!(output, &(from..to+1).collect::<Vec<_>>(), "tree_orig={}", tree_orig);
        check_bst(&tree_mod, &output, tree_orig, 0);
        check_integrity(&tree_mod, &tree_orig);
    }





    quickcheck! {
        fn quickcheck_plain_(xs: Vec<usize>, rm: Range<usize>) -> bool {
            check_plain_tree(xs, rm)
        }
    }

    fn check_plain_tree(mut xs: Vec<usize>, rm: Range<usize>) -> bool {
        xs.sort();
        let rm = if rm.start <= rm.end { rm } else {rm.end .. rm.start};

        let tree = PlainTeardownTree::new(xs);
        check_tree(tree, rm)
    }

    fn check_tree(mut tree: PlainTeardownTree<usize>, rm: Range<usize>) -> bool {
        let tree: &mut TreeWrapper<usize> = tree.internal();
        let orig = tree.clone();

        let mut output = Vec::with_capacity(tree.size());
        tree.delete_range(rm.start, rm.end, &mut output);

        check_bst(&tree, &output, &orig, 0);
        check_integrity(&tree, &orig);

        true
    }
}
