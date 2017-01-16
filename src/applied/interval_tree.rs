use applied::interval::{Interval, IvNode};
use base::{TreeWrapper, TreeBase, Node, KeyVal, BulkDeleteCommon, ItemVisitor, SlotStack, Slot, lefti, righti, parenti};
use base::drivers::{consume_unchecked};
use std::{mem, cmp};
use std::marker::PhantomData;

type IvTree<Iv, V> = TreeWrapper<IvNode<Iv, V>>;

pub trait IntervalTreeInternal<Iv: Interval, V> {
    #[inline] fn delete(&mut self, search: &Iv) -> Option<V>;
    #[inline] fn delete_intersecting(&mut self, search: &Iv, output: &mut Vec<(Iv, V)>);
}


impl<Iv: Interval, V> IntervalTreeInternal<Iv, V> for IvTree<Iv, V> {
    /// Deletes the item with the given key from the tree and returns it (or None).
    #[inline]
    fn delete(&mut self, search: &Iv) -> Option<V> {
        self.index_of(search).map(|idx| {
            let kv = self.delete_idx(idx);
            self.update_ancestors_after_delete(idx, &kv.key.b());
            kv.val
        })
    }

    #[inline]
    fn delete_intersecting(&mut self, search: &Iv, output: &mut Vec<(Iv, V)>) {
        if self.size() != 0 {
            UpdateMax::visit(self, 0, move |this, _|
                this.delete_intersecting_ivl_rec(search, 0, false, output)
            )
        }
    }
}


trait IntervalDelete<Iv: Interval, V>: TreeBase<IvNode<Iv, V>> {
    #[inline]
    fn update_maxb(&mut self, idx: usize) {
        let node = self.node_mut_unsafe(idx);

        let left_self_maxb =
            if self.has_left(idx) {
                cmp::max(&self.left(idx).maxb, node.key.b())
            } else {
                node.key.b()
            }.clone();
        node.maxb =
            if self.has_right(idx) {
                cmp::max(self.right(idx).maxb.clone(), left_self_maxb)
            } else {
                left_self_maxb
            };
    }

    #[inline]
    fn update_ancestors_after_delete(&mut self, mut idx: usize, removed_b: &Iv::K) {
        while idx != 0 {
            idx = parenti(idx);
            if removed_b == &self.node(idx).maxb {
                self.update_maxb(idx);
            } else {
                break;
            }
        }
    }

    #[inline]
    fn delete_idx(&mut self, idx: usize) -> KeyVal<Iv, V> {
        debug_assert!(!self.is_nil(idx));

        let node = self.node_mut_unsafe(idx);

        let repl_kv = match (self.has_left(idx), self.has_right(idx)) {
            (false, false) => {
                let IvNode{kv, ..} = self.take(idx);
                kv
            },

            (true, false)  => {
                let (kv, left_maxb) = self.delete_max(lefti(idx));
                node.maxb = left_maxb;
                kv
            },

            (false, true)  => {
//                let (removed, right_maxb) = self.delete_min(righti(idx));
//                item.maxb = right_maxb;
                let kv = self.delete_min(righti(idx));
                if &node.maxb == kv.key.b() {
                    self.update_maxb(idx)
                } else { // maxb remains the same
                    debug_assert!(kv.key.b() < &node.maxb);
                }
                kv
            },

            (true, true)   => {
                let (kv, left_maxb) = self.delete_max(lefti(idx));
                if &node.maxb == kv.key.b() {
                    node.maxb = cmp::max(left_maxb, self.right(idx).maxb.clone());
                } else { // maxb remains the same
                    debug_assert!(kv.key.b() < &node.maxb);
                }
                kv
            },
        };

        mem::replace(&mut node.kv, repl_kv)
    }


    /// returns the removed max-item of this subtree and the old maxb (before removal)
    #[inline]
    // we attempt to reduce the number of memory accesses as much as possible; might be overengineered
    fn delete_max(&mut self, idx: usize) -> (KeyVal<Iv,V>, Iv::K) {
        let max_idx = self.find_max(idx);

        let (kv, mut old_maxb, mut new_maxb) = if self.has_left(max_idx) {
            let node = self.node_mut_unsafe(max_idx);
            let (left_max_kv, left_maxb) = self.delete_max(lefti(max_idx));
            let kv = mem::replace(&mut node.kv, left_max_kv);

            let old_maxb = mem::replace(&mut node.maxb, left_maxb.clone());
            (kv, old_maxb, Some(left_maxb))
        } else {
            let IvNode { kv, maxb:old_maxb } = self.take(max_idx);
            (kv, old_maxb, None)
        };

        // update ancestors
        let mut upd_idx = max_idx;
        while upd_idx != idx {
            upd_idx = parenti(upd_idx);

            let node = self.node_mut_unsafe(upd_idx);
            old_maxb = node.maxb.clone();
            if &node.maxb == kv.key.b() {
                let mb = {
                    let self_left_maxb =
                        if self.has_left(upd_idx) {
                            cmp::max(&self.left(upd_idx).maxb, &node.maxb)
                        } else {
                            &node.maxb
                        };

                    new_maxb.map_or(self_left_maxb.clone(),
                                    |mb| cmp::max(mb, self_left_maxb.clone()))
                };
                node.maxb = mb.clone();
                new_maxb = Some(mb);
            } else {
                new_maxb = Some(old_maxb.clone());
            }
        }

        (kv, old_maxb)
    }

    // TODO: check whether optimizations similar to delete_max() are worth it
    #[inline]
    fn delete_min(&mut self, idx: usize) -> KeyVal<Iv,V> {
        let min_idx = self.find_min(idx);

        let replacement_kv = if self.has_right(min_idx) {
            let right_min_kv = self.delete_min(righti(min_idx));
            let node = self.node_mut_unsafe(min_idx);

            if self.has_right(min_idx) {
                let right_maxb = &self.right(min_idx).maxb;
                node.maxb = cmp::max(right_maxb, right_min_kv.key.b()).clone();
            } else {
                node.maxb = right_min_kv.key.b().clone();
            }

            mem::replace(&mut node.kv, right_min_kv)
        } else {
            let IvNode{kv, ..} = self.take(min_idx);
            kv
        };

        // update ancestors
        let mut upd_idx = min_idx;
        while upd_idx != idx {
            upd_idx = parenti(upd_idx);
            self.update_maxb(upd_idx);
        }

        replacement_kv
    }
}


trait IntervalDeleteRange<Iv: Interval, V>: BulkDeleteCommon<IvNode<Iv, V>> + IntervalDelete<Iv, V> {
    #[inline(never)]
    fn delete_intersecting_ivl_rec(&mut self, search: &Iv, idx: usize, min_included: bool, output: &mut Vec<(Iv, V)>) {
        let node = self.node_unsafe(idx);
        let k: &Iv = &node.key;

        if &node.maxb < search.a() {
            // whole subtree outside the range
            if self.slots_min().has_open() {
                self.fill_slots_min(idx);
            }
            if self.slots_max().has_open() && !self.is_nil(idx) {
                self.fill_slots_max(idx);
            }
        } else if search.b() <= k.a() && k.a() != search.a() {
            // root and right are outside the range
            self.descend_delete_intersecting_ivl_left(search, idx, false, min_included, output);

            let removed = if self.slots_min().has_open() {
                self.fill_slot_min(idx);

                self.descend_fill_min_right(idx, true)
            } else {
                false
            };

            if self.slots_max().has_open() {
                self.descend_fill_max_left(idx, removed);
            }
        } else {
            // consume root if necessary
            let consumed = if search.intersects(k)
                { Some(self.take(idx)) }
            else
                { None };

            // left subtree
            let mut removed = consumed.is_some();
            if removed {
                if min_included {
                    self.consume_subtree(lefti(idx), output)
                } else {
                    removed = self.descend_delete_intersecting_ivl_left(search, idx, true, false, output);
                }

                consume_unchecked(output, consumed.unwrap().into_kv());
            } else {
                removed = self.descend_delete_intersecting_ivl_left(search, idx, false, min_included, output);
                if !removed && self.slots_min().has_open() {
                    removed = true;
                    self.fill_slot_min(idx);
                }
            }

            // right subtree
            let right_min_included = min_included || search.a() <= k.a();
            if right_min_included {
                let right_max_included = &node.maxb < search.b();
                if right_max_included {
                    self.consume_subtree(righti(idx), output);
                } else {
                    removed = self.descend_delete_intersecting_ivl_right(search, idx, removed, true, output);
                }
            } else {
                removed = self.descend_delete_intersecting_ivl_right(search, idx, removed, false, output);
            }

            if !removed && self.slots_max().has_open() {
                removed = true;
                self.fill_slot_max(idx);
            }

            // fill the remaining open slots_max from the left subtree
            if removed {
                self.descend_fill_max_left(idx, true);
            }
        }
    }


    // Assumes that the returned vec will never be realloc'd!
    #[inline(always)]
    fn pin_stack(stack: &mut SlotStack) -> SlotStack {
        let nslots = stack.nslots;
        let slots = {
            let ptr = stack.slot_at(nslots) as *mut Slot;
            SlotStack {
                nslots: 0,
                nfilled: 0,
                slots: ptr,
                capacity: stack.capacity - nslots
            }
        };

        slots
    }

    /// Returns true if the item is removed after recursive call, false otherwise.
    #[inline(always)]
    fn descend_delete_intersecting_ivl_left(&mut self, search: &Iv, idx: usize, with_slot: bool, min_included: bool, output: &mut Vec<(Iv, V)>) -> bool {
        // this slots_max business is asymmetric (we don't do it in descend_delete_intersecting_ivl_right) because of the program flow: we enter the left subtree first
        let slots_max_left = Self::pin_stack(self.slots_max());
        let slots_max_orig = mem::replace(self.slots_max(), slots_max_left);

        let result = self.descend_left(idx, with_slot,
                          |this: &mut Self, child_idx| this.delete_intersecting_ivl_rec(search, child_idx, min_included, output));

        debug_assert!(self.slots_max().is_empty());
        let slots_max_left = mem::replace(self.slots_max(), slots_max_orig);
        mem::forget(slots_max_left);

        result
    }

    /// Returns true if the item is removed after recursive call, false otherwise.
    #[inline(always)]
    fn descend_delete_intersecting_ivl_right(&mut self, search: &Iv, idx: usize, with_slot: bool, min_included: bool, output: &mut Vec<(Iv, V)>) -> bool {
        self.descend_right(idx, with_slot,
                           |this: &mut Self, child_idx| this.delete_intersecting_ivl_rec(search, child_idx, min_included, output))
    }
}


pub struct UpdateMax;

impl<Iv: Interval, V> ItemVisitor<IvNode<Iv, V>> for UpdateMax {
    type Tree = IvTree<Iv, V>;

    #[inline]
    fn visit<F>(tree: &mut Self::Tree, idx: usize, mut f: F)
                                                    where F: FnMut(&mut Self::Tree, usize) {
        f(tree, idx);

        if tree.is_nil(idx) {
            return;
        }

        let node = &mut tree.node_mut_unsafe(idx);
        match (tree.has_left(idx), tree.has_right(idx)) {
            (false, false) =>
                node.maxb = node.key.b().clone(),
            (false, true) =>
                node.maxb = cmp::max(node.key.b(), &tree.node(righti(idx)).maxb).clone(),
            (true, false) =>
                node.maxb = cmp::max(node.key.b(), &tree.node(lefti(idx)).maxb).clone(),
            (true, true) =>
                node.maxb = cmp::max(node.key.b(),
                                     cmp::max(&tree.node(lefti(idx)).maxb, &tree.node(righti(idx)).maxb))
                                    .clone(),
        }
    }
}


impl<Iv: Interval, V> BulkDeleteCommon<IvNode<Iv, V>> for IvTree<Iv, V> {
    type Visitor = UpdateMax;
}


impl<Iv: Interval, V> IntervalDelete<Iv, V> for IvTree<Iv, V> {}
impl<Iv: Interval, V> IntervalDeleteRange<Iv, V> for IvTree<Iv, V> {}
