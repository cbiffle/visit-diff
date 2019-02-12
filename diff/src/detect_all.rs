use std::fmt::Debug;
use void::{ResultVoidExt, Void};

use crate::{
    Diff, Differ, MapDiffer, SeqDiffer, SetDiffer, StructDiffer, TupleDiffer,
};

use crate::detect::any_difference;

pub fn all_different<T>(a: &T, b: &T) -> bool
where
    T: Diff + ?Sized,
{
    Diff::diff(a, b, Detector).void_unwrap()
}

#[derive(Copy, Clone, Debug)]
struct Detector;

impl Differ for Detector {
    type Ok = bool;
    type Err = Void;

    type StructDiffer = StructDetector;
    type StructVariantDiffer = StructDetector;
    type TupleDiffer = TupleDetector;
    type TupleVariantDiffer = TupleDetector;
    type SeqDiffer = SeqDetector;
    type MapDiffer = MapDetector;
    type SetDiffer = SetDetector;

    fn difference(self, _: &Debug, _: &Debug) -> Result<Self::Ok, Self::Err> {
        Ok(true)
    }

    fn same(self, _: &Debug, _: &Debug) -> Result<Self::Ok, Self::Err> {
        Ok(false)
    }

    fn diff_newtype<T: ?Sized>(
        self,
        _: &'static str,
        a: &T,
        b: &T,
    ) -> Result<Self::Ok, Self::Err>
    where
        T: Diff,
    {
        Diff::diff(a, b, self)
    }

    /// Begin traversing a struct.
    fn begin_struct(self, _: &'static str) -> Self::StructDiffer {
        StructDetector::default()
    }

    fn begin_struct_variant(
        self,
        _: &'static str,
        _: &'static str,
    ) -> Self::StructVariantDiffer {
        StructDetector::default()
    }

    fn begin_tuple(self, _: &'static str) -> Self::TupleDiffer {
        TupleDetector::default()
    }

    fn begin_tuple_variant(
        self,
        _: &'static str,
        _: &'static str,
    ) -> Self::TupleVariantDiffer {
        TupleDetector::default()
    }

    fn begin_seq(self) -> Self::SeqDiffer {
        SeqDetector::default()
    }

    fn begin_map(self) -> Self::MapDiffer {
        MapDetector::default()
    }

    fn begin_set(self) -> Self::SetDiffer {
        SetDetector::default()
    }
}

#[derive(Clone, Debug, Default)]
struct DiffCounter {
    diffs: usize,
    total: usize,
}

impl DiffCounter {
    fn consider<T>(&mut self, a: &T, b: &T)
    where
        T: ?Sized + Diff,
    {
        self.total += 1;
        if any_difference(a, b) {
            self.diffs += 1;
        }
    }

    fn diff(&mut self) {
        self.total += 1;
        self.diffs += 1;
    }

    fn all(self) -> bool {
        self.total == self.diffs
    }
}

#[derive(Clone, Debug, Default)]
struct StructDetector(DiffCounter);

impl StructDiffer for StructDetector {
    type Ok = bool;
    type Err = Void;

    fn diff_field<T: ?Sized>(&mut self, _: &'static str, a: &T, b: &T)
    where
        T: Diff,
    {
        self.0.consider(a, b)
    }

    fn end(self) -> Result<Self::Ok, Self::Err> {
        Ok(self.0.all())
    }
}

#[derive(Clone, Debug, Default)]
struct TupleDetector(DiffCounter);

impl TupleDiffer for TupleDetector {
    type Ok = bool;
    type Err = Void;

    fn diff_field<T: ?Sized>(&mut self, a: &T, b: &T)
    where
        T: Diff,
    {
        self.0.consider(a, b)
    }

    fn end(self) -> Result<Self::Ok, Self::Err> {
        Ok(self.0.all())
    }
}

#[derive(Clone, Debug, Default)]
struct SeqDetector(DiffCounter);

impl SeqDiffer for SeqDetector {
    type Ok = bool;
    type Err = Void;

    fn diff_element<T: ?Sized>(&mut self, a: &T, b: &T)
    where
        T: Diff,
    {
        self.0.consider(a, b)
    }

    fn left_excess<T: ?Sized>(&mut self, _: &T)
    where
        T: Diff,
    {
        self.0.diff()
    }

    fn right_excess<T: ?Sized>(&mut self, _: &T)
    where
        T: Diff,
    {
        self.0.diff()
    }

    fn end(self) -> Result<Self::Ok, Self::Err> {
        Ok(self.0.all())
    }
}

#[derive(Clone, Debug, Default)]
struct SetDetector(DiffCounter);

impl SetDiffer for SetDetector {
    type Ok = bool;
    type Err = Void;

    fn diff_equal<V>(&mut self, a: &V, b: &V)
    where
        V: ?Sized + Diff,
    {
        self.0.consider(a, b)
    }

    fn only_in_left<V>(&mut self, _: &V)
    where
        V: ?Sized + Diff,
    {
        self.0.diff()
    }

    fn only_in_right<V>(&mut self, _: &V)
    where
        V: ?Sized + Diff,
    {
        self.0.diff()
    }

    fn end(self) -> Result<Self::Ok, Self::Err> {
        Ok(self.0.all())
    }
}

#[derive(Clone, Debug, Default)]
struct MapDetector(DiffCounter);

impl MapDiffer for MapDetector {
    type Ok = bool;
    type Err = Void;

    fn diff_entry<K, V>(&mut self, _: &K, a: &V, b: &V)
    where
        K: ?Sized + Debug,
        V: ?Sized + Diff,
    {
        self.0.consider(a, b)
    }

    fn only_in_left<K, V>(&mut self, _: &K, _: &V)
    where
        K: ?Sized + Debug,
        V: ?Sized + Diff,
    {
        self.0.diff()
    }

    fn only_in_right<K, V>(&mut self, _: &K, _: &V)
    where
        K: ?Sized + Debug,
        V: ?Sized + Diff,
    {
        self.0.diff()
    }

    fn end(self) -> Result<Self::Ok, Self::Err> {
        Ok(self.0.all())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::TestStruct;

    #[test]
    fn detector_self_false() {
        let a = TestStruct {
            distance: 12,
            silly: false,
        };
        assert_eq!(all_different(&a, &a), false)
    }

    #[test]
    fn detector_one_field_false() {
        let a = TestStruct {
            distance: 12,
            silly: false,
        };
        let b = TestStruct { distance: 10, ..a };
        assert_eq!(all_different(&a, &b), false)
    }

    #[test]
    fn detector_both_fields_true() {
        let a = TestStruct {
            distance: 12,
            silly: false,
        };
        let b = TestStruct {
            distance: 10,
            silly: true,
        };
        assert!(all_different(&a, &b))
    }
}
