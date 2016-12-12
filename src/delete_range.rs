use base::{TeardownTree, TeardownTreeInternal, Item, Node, lefti, righti};
use drivers::{TraversalDriver, TraversalDecision};
use slot_stack::SlotStack;



pub struct DeleteRangeCache {
    pub slots_min: SlotStack, pub slots_max: SlotStack,   // TODO: we no longer need both stacks, one is enough... but removing one causes 3% performance regression
}

impl Clone for DeleteRangeCache {
    fn clone(&self) -> Self {
        debug_assert!(self.slots_min.is_empty() && self.slots_max.is_empty());
        let capacity = self.slots_max.capacity;
        DeleteRangeCache::new(capacity)
    }
}

impl DeleteRangeCache {
    pub fn new(height: usize) -> DeleteRangeCache {
        let slots_min = SlotStack::new(height);
        let slots_max = SlotStack::new(height);
        DeleteRangeCache { slots_min: slots_min, slots_max: slots_max }
    }
}


pub struct DeleteRange<'a, T: 'a+Item> {
    tree: &'a mut TeardownTree<T>,
    slots_min: SlotStack, slots_max: SlotStack,
    pub output: &'a mut Vec<T>
}

impl<'a, T: Item> DeleteRange<'a, T> {
    pub fn new(tree: &'a mut TeardownTree<T>, output: &'a mut Vec<T>) -> DeleteRange<'a, T> {
        let cache = tree.delete_range_cache.take().unwrap();
        DeleteRange { tree: tree, slots_min: cache.slots_min, slots_max: cache.slots_max, output: output }
    }

    /// The items are returned in order.
    pub fn delete_range<D: TraversalDriver<T::Key>>(mut self, drv: &mut D) {
        self.delete_range_loop(drv, 0);
        debug_assert!(self.slots_min.is_empty() && self.slots_max.is_empty());

        self.tree.delete_range_cache = Some(DeleteRangeCache { slots_min: self.slots_min, slots_max: self.slots_max });
    }


    //---- helpers ---------------------------------------------------------------------------------
    #[inline(always)]
    fn node(&self, idx: usize) -> &Node<T> {
        self.tree.node(idx)
    }

    #[inline(always)]
    fn item(&mut self, idx: usize) -> &T {
        &self.node(idx).item
    }
}


// v5
impl<'a, T: Item> DeleteRange<'a, T> {
    #[inline]
    fn delete_range_loop<D: TraversalDriver<T::Key>>(&mut self, drv: &D, mut idx: usize) {
        loop {
            if self.tree.is_null(idx) {
                return;
            }

            let decision = drv.decide(&self.item(idx).key());

            if decision.left() && decision.right() {
                let item = self.tree.take(idx);
                let removed = self.descend_delete_left(drv, idx, true);
                self.output.push(item);
                self.descend_delete_right(drv, idx, removed);
                return;
            } else if decision.left() {
                idx = lefti(idx);
            } else {
                debug_assert!(decision.right());
                idx = righti(idx);
            }
        }
    }

    #[inline(never)]
    fn delete_range_min<D: TraversalDriver<T::Key>>(&mut self, drv: &D, idx: usize) {
        let decision = drv.decide(&self.item(idx).key());
        debug_assert!(decision.left());

        if decision.right() {
            // the root and the whole left subtree are inside the range
            let item = self.tree.take(idx);
            self.consume_subtree(lefti(idx));
            self.output.push(item);
            self.descend_delete_right(drv, idx, true);
        } else {
            // the root and the right subtree are outside the range
            self.descend_left(idx,
                   |this: &mut Self, child_idx| this.delete_range_min(drv, child_idx)
            );

            if self.slots_min.has_open() {
                self.slots_min.fill(self.tree, idx);
                self.descend_fill_right(idx);
            }
        }
    }

    #[inline(never)]
    fn delete_range_max<D: TraversalDriver<T::Key>>(&mut self, drv: &D, idx: usize) {
        let decision = drv.decide(&self.item(idx).key());
        debug_assert!(decision.right());

        if decision.left() {
            // the root and the whole right subtree are inside the range
            let item = self.tree.take(idx);
            self.descend_delete_left(drv, idx, true);
            self.output.push(item);
            self.consume_subtree(righti(idx));
        } else {
            // the root and the left subtree are outside the range
            self.descend_right(idx,
                   |this: &mut Self, child_idx| this.delete_range_max(drv, child_idx)
            );

            if self.slots_max.has_open() {
                self.slots_max.fill(self.tree, idx);
                self.descend_fill_left(idx);
            }
        }
    }


    //---- delete_subtree_* ---------------------------------------------------------------
    #[inline]
    fn consume_subtree(&mut self, root: usize) {
        self.tree.traverse_inorder(root, self.output, |tree, output, idx| {
            unsafe {
                let len = output.len();
                debug_assert!(len < output.capacity());
                let p = output.as_mut_ptr().offset(len as isize);
                output.set_len(len + 1);
                tree.move_to(idx, p)
            }
        });
    }


    //---- fill_slots_* -------------------------------------------------------------------
    #[inline(never)]
    fn fill_slots_min(&mut self, idx: usize) -> bool {
        debug_assert!(!self.tree.is_null(idx));

        if self.tree.has_left(idx) {
            let done = self.fill_slots_min(lefti(idx));
            if done {
                return true;
            }
        }

        debug_assert!(self.slots_min.has_open());

        self.slots_min.fill(self.tree, idx);

        let done = !self.descend_fill_right(idx);
        done || !self.slots_min.has_open()
    }


    #[inline(never)]
    fn fill_slots_max(&mut self, idx: usize) -> bool {
        debug_assert!(!self.tree.is_null(idx));

        if self.tree.has_right(idx) {
            let done = self.fill_slots_max(righti(idx));
            if done {
                return true;
            }
        }

        debug_assert!(self.slots_max.has_open());

        self.slots_max.fill(self.tree, idx);

        let done = !self.descend_fill_left(idx);
        done || !self.slots_max.has_open()
    }


    //---- descend_* -------------------------------------------------------------------------------


    #[inline(always)]
    fn descend_left<F>(&mut self, idx: usize, f: F)
        where F: Fn(&mut Self, usize) {
        let child_idx = lefti(idx);
        if !self.tree.is_null(child_idx) {
            f(self, child_idx);
        }
    }

    #[inline(always)]
    fn descend_right<F>(&mut self, idx: usize, f: F)
        where F: Fn(&mut Self, usize) {
        let child_idx = righti(idx);
        if !self.tree.is_null(child_idx) {
            f(self, child_idx);
        }
    }


    /// Returns true if the item is removed after recursive call, false otherwise.
    #[inline(always)]
    fn descend_left_with_slot<F>(&mut self, idx: usize, f: F) -> bool
                                                        where F: Fn(&mut Self, usize) {
        debug_assert!(self.tree.is_null(idx));

        let child_idx = lefti(idx);
        if self.tree.is_null(child_idx) {
            return true;
        }

        self.slots_max.push(idx);

        f(self, child_idx);

        self.slots_max.pop();
        self.tree.is_null(idx)
    }

    /// Returns true if the item is removed after recursive call, false otherwise.
    #[inline(always)]
    fn descend_right_with_slot<F>(&mut self, idx: usize, f: F) -> bool
                                                        where F: Fn(&mut Self, usize) {
        debug_assert!(self.tree.is_null(idx));

        let child_idx = righti(idx);
        if self.tree.is_null(child_idx) {
            return true;
        }

        self.slots_min.push(idx);

        f(self, child_idx);

        self.slots_min.pop();
        self.tree.is_null(idx)
    }


    #[inline(always)]
    fn descend_fill_left(&mut self, idx: usize) -> bool {
        self.descend_left_with_slot(idx, |this: &mut Self, child_idx| {
            this.fill_slots_max(child_idx);
        })
    }

    #[inline(always)]
    fn descend_fill_right(&mut self, idx: usize) -> bool {
        self.descend_right_with_slot(idx, |this: &mut Self, child_idx| {
            this.fill_slots_min(child_idx);
        })
    }


    /// Returns true if the item is removed after recursive call, false otherwise.
    #[inline(always)]
    fn descend_delete_left<D: TraversalDriver<T::Key>>(&mut self, drv: &D, idx: usize, with_slot: bool) -> bool {
        if with_slot {
            self.descend_left_with_slot(idx,
                        |this: &mut Self, child_idx| this.delete_range_max(drv, child_idx)
            )
        } else {
            self.descend_left(idx,
                        |this: &mut Self, child_idx| this.delete_range_max(drv, child_idx)
            );

            false
        }
    }

    /// Returns true if the item is removed after recursive call, false otherwise.
    #[inline(always)]
    fn descend_delete_right<D: TraversalDriver<T::Key>>(&mut self, drv: &D, idx: usize, with_slot: bool) -> bool {
        if with_slot {
            self.descend_right_with_slot(idx,
                        |this: &mut Self, child_idx| this.delete_range_min(drv, child_idx)
            )
        } else {
            self.descend_right(idx,
                        |this: &mut Self, child_idx| this.delete_range_min(drv, child_idx)
            );

            false
        }
    }
}
