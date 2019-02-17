use core::fmt::Debug;
use core::marker::PhantomData;
use itertools::{EitherOrBoth, Itertools};
use void::{ResultVoidExt, Void};

use crate::{
    Diff, Differ, MapDiffer, SeqDiffer, SetDiffer, StructDiffer, TupleDiffer,
};

/// Checks for any difference between `a` and `b`.
///
/// This difference could be at the very top (like different variants of an
/// enum) or nested within the structure.
///
/// ```
/// use visit_diff::{Diff, any_difference};
///
/// #[derive(Diff, Debug)]
/// struct ExampleStruct {
///     name: &'static str,
///     age: usize,
/// }
///
/// let left = ExampleStruct { name: "Bob", age: 4 };
/// let right = ExampleStruct { name: "Rototron 3k", age: 5 };
///
/// assert_eq!(any_difference(&left, &left), false);
/// assert_eq!(any_difference(&left, &right), true);
/// ```
pub fn any_difference<T>(a: &T, b: &T) -> bool
where
    T: Diff + ?Sized,
{
    Diff::diff(a, b, Detector::<Any>::default()).void_unwrap()
}

/// Checks if there is something different about *every top-level part* of `a`
/// and `b`. For example, if every field of two structs has some difference.
///
/// The individual parts are compared using [`any_difference`].
///
/// This is used to adjust the granularity of diff reporting by [`debug_diff`]:
/// it will show compound types (like structs) as different at the top level if
/// they are `all_different`, or diff individual fields if not.
///
/// ```
/// use visit_diff::{Diff, all_different};
///
/// #[derive(Diff, Debug)]
/// struct ExampleStruct {
///     name: &'static str,
///     age: usize,
/// }
///
/// let left = ExampleStruct { name: "Bob", age: 4 };
/// let right = ExampleStruct { name: "Rototron 3k", age: 4 };
///
/// assert_eq!(
///     all_different(&left, &right),
///     false,
///     "Bob and Rototron have few things in common, but they're the same age.",
/// );
///
/// let right = ExampleStruct { age: 5, ..right};  // if we change the age...
/// assert_eq!(all_different(&left, &right), true);
/// ```
///
/// [`any_difference`]: fn.any_difference.html
/// [`debug_diff`]: fn.debug_diff.html
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
        T: ?Sized + Diff,
    {
        if !self.0 {
            self.0 = any_difference(a, b);
        }
    }

    fn consider_all<I>(&mut self, left: I, right: I)
    where
        I: IntoIterator,
        I::Item: Diff,
    {
        if !self.0 {
            self.0 = left.into_iter().zip_longest(right).any(|ab| match ab {
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
struct All {
    all: bool,
    any: bool,
}

impl Default for All {
    fn default() -> Self {
        All {
            all: true,
            any: false,
        }
    }
}

impl Accumulator for All {
    fn consider<T>(&mut self, a: &T, b: &T)
    where
        T: ?Sized + Diff,
    {
        if self.all {
            self.all = any_difference(a, b);
            self.any = true;
        }
    }

    fn consider_all<I>(&mut self, left: I, right: I)
    where
        I: IntoIterator,
        I::Item: Diff,
    {
        if self.all {
            *self =
                left.into_iter().zip_longest(right).fold(
                    *self,
                    |s, ab| match ab {
                        EitherOrBoth::Both(a, b) => All {
                            all: s.all && any_difference(&a, &b),
                            any: true,
                        },
                        _ => All {
                            all: false,
                            any: true,
                        },
                    },
                );
        }
    }

    fn diff(&mut self) {
        self.any = true
    }
}

impl From<All> for bool {
    fn from(x: All) -> bool {
        println!("{:?}", x);
        x.any && x.all
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
        assert_eq!(any_difference(&TestEnum::First, &TestEnum::First), false);
        assert_eq!(any_difference(&TestEnum::Second, &TestEnum::Second), false);

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
mod tests {
    use super::*;
    use crate::tests::TestStruct;

    #[test]
    fn all_self_false() {
        let a = TestStruct {
            distance: 12,
            silly: false,
        };
        assert_eq!(all_different(&a, &a), false)
    }

    #[test]
    fn all_one_field_false() {
        let a = TestStruct {
            distance: 12,
            silly: false,
        };
        let b = TestStruct { distance: 10, ..a };
        assert_eq!(all_different(&a, &b), false)
    }

    #[test]
    fn all_both_fields_true() {
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

    #[test]
    fn empty_slice() {
        let s: &[u32] = &[];
        assert!(
            !all_different(&s, &s),
            "empty slices should all be the same."
        );
        assert!(
            !any_difference(&s, &s),
            "empty slices should all be the same."
        );
    }

    #[test]
    fn unit() {
        assert!(!all_different(&(), &()), "units should all be the same.");
        assert!(!any_difference(&(), &()), "units should all be the same.");
    }
}
