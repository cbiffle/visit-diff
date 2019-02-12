use std::fmt::Debug;
use std::marker::PhantomData;
use void::{ResultVoidExt, Void};
use itertools::{Itertools, EitherOrBoth};

use crate::{
    Diff, Differ, MapDiffer, SeqDiffer, SetDiffer, StructDiffer, TupleDiffer,
};

pub fn any_difference<T>(a: &T, b: &T) -> bool
where
    T: Diff + ?Sized,
{
    Diff::diff(a, b, Detector::<Any>::default()).void_unwrap()
}

pub fn all_different<T>(a: &T, b: &T) -> bool
where
    T: Diff + ?Sized,
{
    Diff::diff(a, b, Detector::<All>::default()).void_unwrap()
}

trait Accumulator: Into<bool> + Default {
    fn consider<T>(&mut self, a: &T, b: &T)
    where
        T: ?Sized + Diff;

    fn consider_all<I>(&mut self, left: I, right: I)
    where
        I: IntoIterator,
        I::Item: Diff;

    fn diff(&mut self);
}

#[derive(Copy, Clone, Debug, Default)]
struct Any(bool);

impl Accumulator for Any {
    fn consider<T>(&mut self, a: &T, b: &T)
    where
        T: ?Sized + Diff
    {
        if !self.0 {
            self.0 = any_difference(a, b);
        }
    }

    fn consider_all<I>(&mut self, left: I, right: I)
    where
        I: IntoIterator,
        I::Item: Diff
    {
        if !self.0 {
            self.0 = left.into_iter().zip_longest(right)
                .any(|ab| match ab {
                    EitherOrBoth::Both(a, b) => any_difference(&a, &b),
                    _ => true,
                });
        }
    }

    fn diff(&mut self) {
        self.0 = true
    }
}

impl From<Any> for bool {
    fn from(x: Any) -> bool {
        x.0
    }
}

#[derive(Copy, Clone, Debug)]
struct All(bool);

impl Default for All {
    fn default() -> Self {
        All(true)
    }
}

impl Accumulator for All {
    fn consider<T>(&mut self, a: &T, b: &T)
    where
        T: ?Sized + Diff
    {
        if self.0 {
            self.0 = any_difference(a, b);
        }
    }

    fn consider_all<I>(&mut self, left: I, right: I)
    where
        I: IntoIterator,
        I::Item: Diff
    {
        if !self.0 {
            self.0 = left.into_iter().zip_longest(right)
                .all(|ab| match ab {
                    EitherOrBoth::Both(a, b) => any_difference(&a, &b),
                    _ => true,
                });
        }
    }

    fn diff(&mut self) {
        // Doesn't change the result one way or another.
    }
}

impl From<All> for bool {
    fn from(x: All) -> bool {
        x.0
    }
}

#[derive(Copy, Clone, Debug, Default)]
struct Detector<A>(PhantomData<A>);

impl<A: Accumulator> Differ for Detector<A> {
    type Ok = bool;
    type Err = Void;

    type StructDiffer = StructDetector<A>;
    type StructVariantDiffer = StructDetector<A>;
    type TupleDiffer = TupleDetector<A>;
    type TupleVariantDiffer = TupleDetector<A>;
    type SeqDiffer = SeqDetector<A>;
    type MapDiffer = MapDetector<A>;
    type SetDiffer = SetDetector<A>;

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
struct StructDetector<A>(A);

impl<A: Accumulator> StructDiffer for StructDetector<A> {
    type Ok = bool;
    type Err = Void;

    fn diff_field<T: ?Sized>(&mut self, _: &'static str, a: &T, b: &T)
    where
        T: Diff,
    {
        self.0.consider(a, b);
    }

    fn end(self) -> Result<Self::Ok, Self::Err> {
        Ok(self.0.into())
    }
}

#[derive(Clone, Debug, Default)]
struct TupleDetector<A>(A);

impl<A: Accumulator> TupleDiffer for TupleDetector<A> {
    type Ok = bool;
    type Err = Void;

    fn diff_field<T: ?Sized>(&mut self, a: &T, b: &T)
    where
        T: Diff,
    {
        self.0.consider(a, b);
    }

    fn end(self) -> Result<Self::Ok, Self::Err> {
        Ok(self.0.into())
    }
}

#[derive(Clone, Debug, Default)]
struct SeqDetector<A>(A);

impl<A: Accumulator> SeqDiffer for SeqDetector<A> {
    type Ok = bool;
    type Err = Void;

    fn diff_element<T: ?Sized>(&mut self, a: &T, b: &T)
    where
        T: Diff,
    {
        self.0.consider(a, b);
    }

    fn diff_elements<T, I>(&mut self, a: I, b: I)
    where
        T: Diff,
        I: IntoIterator<Item = T>,
    {
        self.0.consider_all(a, b)
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
        Ok(self.0.into())
    }
}

#[derive(Clone, Debug, Default)]
struct SetDetector<A>(A);

impl<A: Accumulator> SetDiffer for SetDetector<A> {
    type Ok = bool;
    type Err = Void;

    fn diff_equal<V>(&mut self, a: &V, b: &V)
    where
        V: ?Sized + Diff,
    {
        self.0.consider(a, b);
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
        Ok(self.0.into())
    }
}

#[derive(Clone, Debug, Default)]
struct MapDetector<A>(A);

impl<A: Accumulator> MapDiffer for MapDetector<A> {
    type Ok = bool;
    type Err = Void;

    fn diff_entry<K, V>(&mut self, _: &K, a: &V, b: &V)
    where
        K: ?Sized + Debug,
        V: ?Sized + Diff,
    {
        self.0.consider(a, b);
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
        Ok(self.0.into())
    }
}

#[cfg(test)]
mod any_tests {
    use super::*;
    use crate::tests::{TestEnum, TestStruct};

    #[test]
    fn detector_self_false() {
        let a = TestStruct {
            distance: 12,
            silly: false,
        };
        assert_eq!(any_difference(&a, &a), false)
    }

    #[test]
    fn detector_other_false() {
        let a = TestStruct {
            distance: 12,
            silly: false,
        };
        assert_eq!(any_difference(&a, &a.clone()), false)
    }

    #[test]
    fn detector_first_field_true() {
        let a = TestStruct {
            distance: 12,
            silly: false,
        };
        let b = TestStruct {
            distance: 10,
            silly: false,
        };
        assert!(any_difference(&a, &b))
    }

    #[test]
    fn detector_second_field_true() {
        let a = TestStruct {
            distance: 12,
            silly: false,
        };
        let b = TestStruct {
            distance: 12,
            silly: true,
        };
        assert!(any_difference(&a, &b))
    }

    #[test]
    fn detector_enum() {
        assert_eq!(
            any_difference(&TestEnum::First, &TestEnum::First),
            false
        );
        assert_eq!(
            any_difference(&TestEnum::Second, &TestEnum::Second),
            false
        );

        assert!(any_difference(&TestEnum::First, &TestEnum::Second));
        assert!(any_difference(&TestEnum::Second, &TestEnum::First));
    }

    #[test]
    fn detector_struct_variant() {
        let a = TestEnum::Struct { a: 12, b: false };

        assert_eq!(any_difference(&a, &a), false);
        assert!(any_difference(&a, &TestEnum::First));

        let b = TestEnum::Struct { a: 14, b: true };

        assert!(any_difference(&a, &b));
    }
}

#[cfg(test)]
mod all_tests {
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
