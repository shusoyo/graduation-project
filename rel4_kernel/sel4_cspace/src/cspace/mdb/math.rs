use vstd::prelude::*;

verus! {

pub proof fn lemma_seq_insert_shift<A>(old_seq: Seq<A>, idx: int, val: A)
    requires
        0 <= idx <= old_seq.len(),
    ensures
        old_seq.insert(idx, val).len() == old_seq.len() + 1,
        forall|i: int| 0 <= i < idx ==> #[trigger] old_seq.insert(idx, val)[i] == old_seq[i],
        old_seq.insert(idx, val)[idx] == val,
        forall|i: int|
            idx < i < old_seq.insert(idx, val).len() ==> #[trigger] old_seq.insert(idx, val)[i]
                == old_seq[i - 1],
{
    old_seq.insert_ensures(idx, val);
}

pub proof fn lemma_seq_remove_shift<A>(old_seq: Seq<A>, idx: int)
    requires
        0 <= idx < old_seq.len(),
    ensures
        old_seq.remove(idx).len() == old_seq.len() - 1,
        forall|i: int| 0 <= i < idx ==> #[trigger] old_seq.remove(idx)[i] == old_seq[i],
        forall|i: int|
            idx <= i < old_seq.remove(idx).len() ==> #[trigger] old_seq.remove(idx)[i]
                == old_seq[i + 1],
{
    old_seq.remove_ensures(idx);
}

}
