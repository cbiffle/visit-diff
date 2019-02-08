pub mod debug;
pub mod detect;

use std::fmt::Debug;

trait Diff {
    fn diff<D>(a: &Self, b: &Self, out: D) -> Result<D::Ok, D::Err>
        where D: Differ;
}

trait Differ {
    type Ok;
    type Err;

    type StructDiffer: StructDiffer<Ok = Self::Ok, Err = Self::Err>;
    type StructVariantDiffer: StructDiffer<Ok = Self::Ok, Err = Self::Err>;
    /*
    type TupleStructDiffer: Differ;
    type TupleVariantDiffer: Differ;
    type TupleDiffer: Differ;
    type SeqDiffer: Differ;
    type MapDiffer: Differ;
    type SetDiffer: Differ;
    */

    /// Two atomic values are different.
    fn difference(self, a: &Debug, b: &Debug) -> Result<Self::Ok, Self::Err>;

    /// Two atomic values are the same.
    fn same(self, a: &Debug, b: &Debug) -> Result<Self::Ok, Self::Err>;

    /// Descend into a newtype.
    fn diff_newtype<T: ?Sized>(self, ty: &'static str, a: &T, b: &T)
        -> Result<Self::Ok, Self::Err>
    where T: Diff;

    /// Begin traversing a struct.
    fn begin_struct(self, ty: &'static str) -> Self::StructDiffer;

    /// Begin traversing a struct variant.
    fn begin_struct_variant(self, ty: &'static str, var: &'static str)
        -> Self::StructVariantDiffer;

    /*
    /// Begin traversing a tuple struct.
    fn begin_tuple_struct(self, ty: &'static str) -> Self::TupleStructDiffer;

    /// Begin traversing a tuple variant.
    fn begin_tuple_variant(self, ty: &'static str, var: &'static str)
        -> Self::TupleVariantDiffer;

    /// Begin traversing a tuple.
    fn begin_tuple(self) -> Self::TupleDiffer;

    /// Begin traversing a sequence.
    fn begin_seq(self) -> Self::SeqDiffer;

    /// Begin traversing a map.
    fn begin_map(self) -> Self::MapDiffer;

    /// Begin traversing a set.
    fn begin_set(self) -> Self::SetDiffer;
    */
}

trait StructDiffer {
    type Ok;
    type Err;

    fn diff_field<T: ?Sized>(&mut self, name: &'static str, a: &T, b: &T)
    where T: Diff;

    fn skip_field<T: ?Sized>(&mut self, _name: &'static str) {}

    fn end(self) -> Result<Self::Ok, Self::Err>;
}

//////////////////////////////////////////////////////////////
// Impls

impl Diff for bool {
    fn diff<D>(a: &Self, b: &Self, out: D) -> Result<D::Ok, D::Err>
        where D: Differ,
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
        where D: Differ,
    {
        if a != b {
            out.difference(a, b)
        } else {
            out.same(a, b)
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug)]
    pub enum TestEnum {
        First,
        Second,
    }

    impl Diff for TestEnum {
        fn diff<D>(a: &Self, b: &Self, out: D) -> Result<D::Ok, D::Err>
        where D: Differ,
        {
            match (a, b) {
                (TestEnum::First, TestEnum::First) => out.same(a, b),
                (TestEnum::Second, TestEnum::Second) => out.same(a, b),
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
        where D: Differ,
        {
            let mut s = out.begin_struct("TestStruct");
            s.diff_field("distance", &a.distance, &b.distance);
            s.diff_field("silly", &a.silly, &b.silly);
            s.end()
        }
    }
}
