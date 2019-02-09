//! Analyzing structural differences in Rust values.
//!
//! This scheme is modeled after a combination of `std::fmt::Formatter` and
//! `serde::Serialize`.

pub mod debug;
pub mod detect;

use std::fmt::Debug;

/// A type that can be compared structurally to discover differences.
pub trait Diff: Debug {
    /// Traverse `a` and `b`, reporting differences to `out`.
    fn diff<D>(a: &Self, b: &Self, out: D) -> Result<D::Ok, D::Err>
    where
        D: Differ;
}

/// A type that can do something with information about structural differences.
pub trait Differ {
    /// Type returned on success.
    type Ok;
    /// Type returned on failure.
    type Err;

    type StructDiffer: StructDiffer<Ok = Self::Ok, Err = Self::Err>;
    type StructVariantDiffer: StructDiffer<Ok = Self::Ok, Err = Self::Err>;
    type TupleDiffer: TupleDiffer<Ok = Self::Ok, Err = Self::Err>;
    type TupleVariantDiffer: TupleDiffer<Ok = Self::Ok, Err = Self::Err>;
    type SeqDiffer: SeqDiffer<Ok = Self::Ok, Err = Self::Err>;
    type MapDiffer: MapDiffer<Ok = Self::Ok, Err = Self::Err>;
    type SetDiffer: SetDiffer<Ok = Self::Ok, Err = Self::Err>;

    /// Two atomic values have been discovered to be different.
    fn difference(self, a: &Debug, b: &Debug) -> Result<Self::Ok, Self::Err>;

    /// Two atomic values are the same.
    fn same(self, a: &Debug, b: &Debug) -> Result<Self::Ok, Self::Err>;

    /// Encounter a newtype.
    fn diff_newtype<T: ?Sized>(
        self,
        ty: &'static str,
        a: &T,
        b: &T,
    ) -> Result<Self::Ok, Self::Err>
    where
        T: Diff;

    /// Begin traversing a struct.
    fn begin_struct(self, ty: &'static str) -> Self::StructDiffer;

    /// Begin traversing a struct variant.
    fn begin_struct_variant(
        self,
        ty: &'static str,
        var: &'static str,
    ) -> Self::StructVariantDiffer;

    /// Begin traversing a tuple struct.
    fn begin_tuple(self, ty: &'static str) -> Self::TupleDiffer;

    /// Begin traversing a tuple variant.
    fn begin_tuple_variant(
        self,
        ty: &'static str,
        var: &'static str,
    ) -> Self::TupleVariantDiffer;

    /// Begin traversing a sequence.
    fn begin_seq(self) -> Self::SeqDiffer;

    /// Begin traversing a map.
    fn begin_map(self) -> Self::MapDiffer;

    /// Begin traversing a set.
    fn begin_set(self) -> Self::SetDiffer;
}

/// A type that can deal with differences in a `struct`.
pub trait StructDiffer {
    type Ok;
    type Err;

    /// Visits a field with values `a` and `b` in the respective structures.
    fn diff_field<T: ?Sized>(&mut self, name: &'static str, a: &T, b: &T)
    where
        T: Diff;

    /// Skips a field that is excluded from differencing.
    fn skip_field<T: ?Sized>(&mut self, _name: &'static str) {}

    /// Completes traversal of the struct.
    fn end(self) -> Result<Self::Ok, Self::Err>;
}

pub trait TupleDiffer {
    type Ok;
    type Err;

    fn diff_field<T: ?Sized>(&mut self, a: &T, b: &T)
    where
        T: Diff;

    fn skip_field<T: ?Sized>(&mut self) {}

    fn end(self) -> Result<Self::Ok, Self::Err>;
}

pub trait SeqDiffer {
    type Ok;
    type Err;

    fn diff_element<T: ?Sized>(&mut self, a: &T, b: &T)
    where
        T: Diff;

    fn diff_elements<T, I>(&mut self, a: I, b: I)
    where
        T: Diff,
        I: IntoIterator<Item = T>,
    {
        for (a, b) in a.into_iter().zip(b.into_iter()) {
            self.diff_element(&a, &b);
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Err>;
}

pub trait MapDiffer {
    type Ok;
    type Err;

    /// Both maps contain entries for `key`; check them for differences.
    fn diff_entry<K, V>(&mut self, key: &K, a: &V, b: &V)
    where
        K: ?Sized + Debug,
        V: ?Sized + Diff;

    /// Key `key` is only present in the left map, with value `a`.
    fn only_in_left<K, V>(&mut self, key: &K, a: &V)
    where
        K: ?Sized + Debug,
        V: ?Sized + Diff;

    /// Key `key` is only present in the right map, with value `b`.
    fn only_in_right<K, V>(&mut self, key: &K, b: &V)
    where
        K: ?Sized + Debug,
        V: ?Sized + Diff;

    /// We've reached the end of the maps.
    fn end(self) -> Result<Self::Ok, Self::Err>;
}

pub trait SetDiffer {
    type Ok;
    type Err;

    /// The sets contain `a` and `b` which compare as equal. Check them for
    /// differences.
    fn diff_equal<V>(&mut self, a: &V, b: &V)
    where
        V: ?Sized + Diff;

    /// Value `a` is only in the left-hand set.
    fn only_in_left<V>(&mut self, a: &V)
    where
        V: ?Sized + Diff;

    /// Value `b` is only in the right-hand set.
    fn only_in_right<V>(&mut self, b: &V)
    where
        V: ?Sized + Diff;

    /// We've reached the end of the sets.
    fn end(self) -> Result<Self::Ok, Self::Err>;
}

//////////////////////////////////////////////////////////////
// Impls

impl<T> Diff for &T
where
    T: Diff,
{
    fn diff<D>(a: &Self, b: &Self, out: D) -> Result<D::Ok, D::Err>
    where
        D: Differ,
    {
        Diff::diff(*a, *b, out)
    }
}

impl Diff for bool {
    fn diff<D>(a: &Self, b: &Self, out: D) -> Result<D::Ok, D::Err>
    where
        D: Differ,
    {
        if a != b {
            out.difference(a, b)
        } else {
            out.same(a, b)
        }
    }
}

impl Diff for usize {
    fn diff<D>(a: &Self, b: &Self, out: D) -> Result<D::Ok, D::Err>
    where
        D: Differ,
    {
        if a != b {
            out.difference(a, b)
        } else {
            out.same(a, b)
        }
    }
}

impl<K, V> Diff for std::collections::BTreeMap<K, V>
where
    K: Ord + Debug,
    V: Diff,
{
    fn diff<D>(a: &Self, b: &Self, out: D) -> Result<D::Ok, D::Err>
    where
        D: Differ,
    {
        use std::cmp::Ordering;

        let mut out = out.begin_map();

        let mut akeys = a.keys().peekable();
        let mut bkeys = b.keys().peekable();

        while let (Some(ka), Some(kb)) = (akeys.peek(), bkeys.peek()) {
            match ka.cmp(kb) {
                Ordering::Less => {
                    out.only_in_left(ka, &a[ka]);
                    akeys.next();
                }
                Ordering::Equal => {
                    out.diff_entry(ka, &a[ka], &b[kb]);
                    akeys.next();
                    bkeys.next();
                }
                Ordering::Greater => {
                    out.only_in_right(kb, &b[kb]);
                    bkeys.next();
                }
            }
        }

        for k in akeys {
            out.only_in_left(k, &a[k])
        }
        for k in bkeys {
            out.only_in_right(k, &b[k])
        }

        out.end()
    }
}

impl<K> Diff for std::collections::BTreeSet<K>
where
    K: Ord + Diff,
{
    fn diff<D>(a: &Self, b: &Self, out: D) -> Result<D::Ok, D::Err>
    where
        D: Differ,
    {
        use std::cmp::Ordering;

        let mut out = out.begin_set();

        let mut akeys = a.iter().peekable();
        let mut bkeys = b.iter().peekable();

        while let (Some(ka), Some(kb)) = (akeys.peek(), bkeys.peek()) {
            match ka.cmp(kb) {
                Ordering::Less => {
                    out.only_in_left(ka);
                    akeys.next();
                }
                Ordering::Equal => {
                    out.diff_equal(ka, kb);
                    akeys.next();
                    bkeys.next();
                }
                Ordering::Greater => {
                    out.only_in_right(kb);
                    bkeys.next();
                }
            }
        }

        for k in akeys {
            out.only_in_left(k)
        }
        for k in bkeys {
            out.only_in_right(k)
        }

        out.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug)]
    pub enum TestEnum {
        First,
        Second,
        Struct { a: usize, b: bool },
    }

    impl Diff for TestEnum {
        fn diff<D>(a: &Self, b: &Self, out: D) -> Result<D::Ok, D::Err>
        where
            D: Differ,
        {
            match (a, b) {
                (TestEnum::First, TestEnum::First) => out.same(a, b),
                (TestEnum::Second, TestEnum::Second) => out.same(a, b),
                (
                    TestEnum::Struct { a: aa, b: ab },
                    TestEnum::Struct { a: ba, b: bb },
                ) => {
                    let mut s = out.begin_struct_variant("TestEnum", "Struct");
                    s.diff_field("a", &aa, &ba);
                    s.diff_field("b", &ab, &bb);
                    s.end()
                }
                _ => out.difference(a, b),
            }
        }
    }

    #[derive(Clone, Debug)]
    pub struct TestStruct {
        pub distance: usize,
        pub silly: bool,
    }

    impl Diff for TestStruct {
        fn diff<D>(a: &Self, b: &Self, out: D) -> Result<D::Ok, D::Err>
        where
            D: Differ,
        {
            let mut s = out.begin_struct("TestStruct");
            s.diff_field("distance", &a.distance, &b.distance);
            s.diff_field("silly", &a.silly, &b.silly);
            s.end()
        }
    }
}
