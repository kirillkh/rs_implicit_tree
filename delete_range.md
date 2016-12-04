`delete_range`
--------------

There are many cases where we might wish to delete (and return) a consecutive
range of values from a binary search tree (BST), but somehow it seems to
be overlooked in the literature. I was able to find a single reference
to such an algorithm, namely the combination of `split`/`merge` functions
as implemented in the [Set module][1] (which is an AVL tree under the hood)
of the OCaml standard library, which both work in `O(k + log n)` time (where `k`
is the number of deleted elements).

However, I cannot make use of `split`/`merge` in a straightforward manner
in this project, considering that it requires explicit detaching and recombining
subtrees, which takes `O(n)` time with an implicit tree representation.
While my case is admittedly niche, it is still surprising that I have not
been able to find any other description of a `delete_range` algorithm on
the web.

This document describes a `delete_range` algorithm that can be used in
any BST, not just in the implicit variant. It is quite a simple idea, really.
Can I be the first one to think of it?

Algorithm
---------

We approach the problem in a direct manner. The algorithm is an extension
of the standard BST `delete` operation. We navigate the tree looking for
elements that are inside the requested range. Given a node `X` and its item
`V`, where `V` is in the search range, we aim to replace it with another
item `W`, such that either either `W = max(left(X))` or `W = min(right(X))`,
where `min(T)` and `max(T)` denote the minimal and maximal items in subtree
`T`.

In order to achieve this, we maintain two stacks: `slots_min` and `slots_max`.
A `slot` is a memory cell which admits two possible values: `Filled(item)`
or `Empty`. The stacks support the following operations:

1. `push(slot)`
2. `pop() -> slot`
3. `fill(item)` - replaces the **deepest** `Empty` slot with `Filled(item)`.

Both stacks are initially empty. For node `X` above, we would
do the following:

1. Move `V` from `X` to the output.
1. Push an `Empty` slot on top of `slots_max`.
1. Recursively call `delete_range(left(X))`.
1. let `slot := slots_max.pop()`
1. If `slot` is `Filled(W)`, then replace `V` with `W`.
1. otherwise, push `Empty` on top of `slots_min` and process the right
   subtree in a similar manner.

Another important case is when the node `X` is **not** inside the query
range. In this case, the idea is to determine whether `item(X) = min(X)`.
If that is the case and `slots_min` has a non-empty slot, we move `item(X)`
to the deepest open slot. Otherwise, we try with `max(X)` and `slots_max`.
Algorithm:

1. Recursively call `delete_range(left(X))`.
2. If `slots_min` has an empty slot, we know that the left subtree is now empty,
   and `item(X) = min(X)`. So we fill a slot in `slots_min` with `item(X)`.
3. If `item(X)` is empty (i.e. was removed in the previous step), we push
   `Empty` on top of `slots_min`.
4. Recursively call `delete_range(right(X))`.
5. If we pushed a slot in the step `(3)`, pop a slot and use its content
   (if non-`Empty`) to replace `item(X)`.
6. If `slots_max` has an empty slot, we know that the right subtree is now 
   empty. However, the left subtree might be non-empty, so we proceed to fill
   the remaining open `slots_max` with items from the left subtree:
   1. if `item(X)` is non-`Empty`, we use it to fill a slot in `slots_max`
      and push `Empty` onto `slots_max`
   1. call `delete_range(left(X))` again
   1. if we pushed a slot in step `(i)`, we pop a slot from `slots_max`
      and use its value to fill `item(X)`


The two algorithms above are, of course, only sketches, and the actual implementation
has more details to consider. See [`delete_range.rs`][2] for the full
breakdown. It is also worth mentioning that the implementation can be significantly
sped up by splitting `delete_range` into separate functions based on the
following cases:

1. The whole subtree is inside the search range. This means we can just
   traverse the subtree, moving every element to the output. Nothing else
   needs to be done.
1. The whole subtree is outside the search range. This means we are only
   here to fill slots in `slots_min` or `slots_max` (we never need to fill
   both at the same time). Write specialized subroutines `fill_slots_min`
   and `fill_slots_max` for this task.
1. Use the general algorithm in the other cases.

**TODO**: implement the algorithm for a normal BST with explicit representation
(child node pointers) and compare against split/merge.


[1]: https://github.com/ocaml/ocaml/blob/trunk/stdlib/set.ml
[2]: https://github.com/kirillkh/rs_teardown_tree/blob/master/src/delete_range.rs
