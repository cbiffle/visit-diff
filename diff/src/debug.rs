//! Report differences using `Debug` and `Formatter`.

use crate::{
    Diff, Differ, MapDiffer, SeqDiffer, SetDiffer, StructDiffer, TupleDiffer,
};
use core::fmt::Debug;

use super::detect::all_different;

/// Adapts a `core::fmt::Formatter` into a `Differ`.
struct DebugDiffer<'a, 'b>(&'a mut core::fmt::Formatter<'b>);

impl<'a, 'b> Differ for DebugDiffer<'a, 'b> {
    type Ok = ();
    type Err = core::fmt::Error;

    type StructDiffer = DebugStructDiff<'a, 'b>;
    type StructVariantDiffer = DebugStructDiff<'a, 'b>;
    type TupleDiffer = DebugTupleDiff<'a, 'b>;
    type TupleVariantDiffer = DebugTupleDiff<'a, 'b>;
    type SeqDiffer = DebugSeqDiff<'a, 'b>;
    type MapDiffer = DebugMapDiff<'a, 'b>;
    type SetDiffer = DebugSetDiff<'a, 'b>;

    fn difference(self, a: &Debug, b: &Debug) -> Result<Self::Ok, Self::Err> {
        DIFF { L: a, R: b }.fmt(self.0)
    }

    fn same(self, a: &Debug, _: &Debug) -> Result<Self::Ok, Self::Err> {
        a.fmt(self.0)
    }

    fn diff_newtype<T: ?Sized>(
        self,
        name: &'static str,
        a: &T,
        b: &T,
    ) -> Result<Self::Ok, Self::Err>
    where
        T: Diff,
    {
        self.0.debug_tuple(name).field(&DebugDiff(a, b)).finish()
    }

    fn begin_struct(self, name: &'static str) -> Self::StructDiffer {
        DebugStructDiff(Ok(self.0.debug_struct(name)))
    }

    fn begin_struct_variant(
        self,
        _: &'static str,
        v: &'static str,
    ) -> Self::StructVariantDiffer {
        DebugStructDiff(Ok(self.0.debug_struct(v)))
    }

    fn begin_tuple(self, ty: &'static str) -> Self::TupleDiffer {
        DebugTupleDiff(Ok(self.0.debug_tuple(ty)))
    }

    fn begin_tuple_variant(
        self,
        _: &'static str,
        v: &'static str,
    ) -> Self::TupleDiffer {
        DebugTupleDiff(Ok(self.0.debug_tuple(v)))
    }

    fn begin_seq(self) -> Self::SeqDiffer {
        DebugSeqDiff(Ok(self.0.debug_list()))
    }

    fn begin_map(self) -> Self::MapDiffer {
        DebugMapDiff(Ok(self.0.debug_map()))
    }

    fn begin_set(self) -> Self::SetDiffer {
        DebugSetDiff(Ok(self.0.debug_set()))
    }
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
struct DIFF<T, S> {
    L: T,
    R: S,
}

struct Missing;

impl core::fmt::Debug for Missing {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        fmt.write_str("(missing)")
    }
}

struct DebugStructDiff<'a, 'b>(
    Result<core::fmt::DebugStruct<'a, 'b>, core::fmt::Error>,
);

impl<'a, 'b> StructDiffer for DebugStructDiff<'a, 'b> {
    type Ok = ();
    type Err = core::fmt::Error;

    fn diff_field<T: ?Sized>(&mut self, name: &'static str, a: &T, b: &T)
    where
        T: Diff,
    {
        if let Ok(f) = &mut self.0 {
            f.field(name, &DebugDiff(a, b) as &dyn core::fmt::Debug);
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Err> {
        self.0.and_then(|mut f| f.finish())
    }
}

struct DebugTupleDiff<'a, 'b>(
    Result<core::fmt::DebugTuple<'a, 'b>, core::fmt::Error>,
);

impl<'a, 'b> TupleDiffer for DebugTupleDiff<'a, 'b> {
    type Ok = ();
    type Err = core::fmt::Error;

    fn diff_field<T: ?Sized>(&mut self, a: &T, b: &T)
    where
        T: Diff,
    {
        if let Ok(f) = &mut self.0 {
            f.field(&DebugDiff(a, b) as &dyn core::fmt::Debug);
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Err> {
        self.0.and_then(|mut f| f.finish())
    }
}

struct DebugSeqDiff<'a, 'b>(
    Result<core::fmt::DebugList<'a, 'b>, core::fmt::Error>,
);

impl<'a, 'b> SeqDiffer for DebugSeqDiff<'a, 'b> {
    type Ok = ();
    type Err = core::fmt::Error;

    fn diff_element<T: ?Sized>(&mut self, a: &T, b: &T)
    where
        T: Diff,
    {
        if let Ok(f) = &mut self.0 {
            f.entry(&DebugDiff(a, b) as &dyn core::fmt::Debug);
        }
    }

    fn left_excess<T: ?Sized>(&mut self, a: &T)
    where
        T: Diff,
    {
        if let Ok(f) = &mut self.0 {
            f.entry(&DIFF { L: a, R: Missing });
        }
    }

    fn right_excess<T: ?Sized>(&mut self, b: &T)
    where
        T: Diff,
    {
        if let Ok(f) = &mut self.0 {
            f.entry(&DIFF { L: Missing, R: b });
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Err> {
        self.0.and_then(|mut f| f.finish())
    }
}

struct DebugSetDiff<'a, 'b>(
    Result<core::fmt::DebugSet<'a, 'b>, core::fmt::Error>,
);

impl<'a, 'b> SetDiffer for DebugSetDiff<'a, 'b> {
    type Ok = ();
    type Err = core::fmt::Error;

    fn diff_equal<V>(&mut self, a: &V, b: &V)
    where
        V: ?Sized + Diff,
    {
        if let Ok(f) = &mut self.0 {
            f.entry(&DebugDiff(a, b) as &dyn core::fmt::Debug);
        }
    }

    fn only_in_left<V>(&mut self, a: &V)
    where
        V: ?Sized + Diff,
    {
        if let Ok(f) = &mut self.0 {
            f.entry(&DIFF { L: a, R: Missing });
        }
    }

    fn only_in_right<V>(&mut self, a: &V)
    where
        V: ?Sized + Diff,
    {
        if let Ok(f) = &mut self.0 {
            f.entry(&DIFF { L: Missing, R: a });
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Err> {
        self.0.and_then(|mut f| f.finish())
    }
}

struct DebugMapDiff<'a, 'b>(
    Result<core::fmt::DebugMap<'a, 'b>, core::fmt::Error>,
);

impl<'a, 'b> MapDiffer for DebugMapDiff<'a, 'b> {
    type Ok = ();
    type Err = core::fmt::Error;

    fn diff_entry<K, V>(&mut self, k: &K, a: &V, b: &V)
    where
        K: ?Sized + Debug,
        V: ?Sized + Diff,
    {
        if let Ok(f) = &mut self.0 {
            f.entry(&k, &DebugDiff(a, b) as &dyn core::fmt::Debug);
        }
    }

    fn only_in_left<K, V>(&mut self, k: &K, a: &V)
    where
        K: ?Sized + Debug,
        V: ?Sized + Diff,
    {
        if let Ok(f) = &mut self.0 {
            f.entry(&k, &DIFF { L: a, R: Missing });
        }
    }

    fn only_in_right<K, V>(&mut self, k: &K, a: &V)
    where
        K: ?Sized + Debug,
        V: ?Sized + Diff,
    {
        if let Ok(f) = &mut self.0 {
            f.entry(&k, &DIFF { L: Missing, R: a });
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Err> {
        self.0.and_then(|mut f| f.finish())
    }
}

/// Wraps a pair of values into an object that, when formatted using `Debug`,
/// shows the differences between the values.
struct DebugDiff<T>(pub T, pub T);

impl<T> core::fmt::Debug for DebugDiff<T>
where
    T: Diff,
{
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        if all_different(&self.0, &self.1) {
            DebugDiffer(fmt).difference(&self.0, &self.1)
        } else {
            Diff::diff(&self.0, &self.1, DebugDiffer(fmt))
        }
    }
}

/// Given two values that can be diffed, returns an object that will describe
/// their differences when formatted using `Debug`.
///
/// The result can be used anywhere you'd normally format something with
/// `Debug`, such as calls to `println!`:
///
/// ```
/// use visit_diff::{Diff, debug_diff};
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
/// println!("Compact: {:?}", debug_diff(&left, &right));
/// println!("Pretty: {:#?}", debug_diff(&left, &right));
/// ```
///
/// This prints:
///
/// ```text
/// Compact: DIFF { L: ExampleStruct { name: "Bob", age: 4 }, R: ExampleStruct { name: "Rototron 3k", age: 5 } }
/// Pretty: DIFF {
///     L: ExampleStruct {
///         name: "Bob",
///         age: 4
///     },
///     R: ExampleStruct {
///         name: "Rototron 3k",
///         age: 5
///     }
/// }
/// ```
///
/// This is showing the `DIFF` at the top level, because all fields of the
/// structs are different. If only a single field is different, a more precise
/// diff happens:
///
/// ```
/// # use visit_diff::{Diff, debug_diff};
/// # #[derive(Diff, Debug)]
/// # struct ExampleStruct {
/// #     name: &'static str,
/// #     age: usize,
/// #  }
/// let left = ExampleStruct { name: "Bob", age: 4 };
/// let right = ExampleStruct { name: "Bob", age: 5 };
///
/// println!("Compact: {:?}", debug_diff(&left, &right));
/// println!("Pretty: {:#?}", debug_diff(&left, &right));
/// ```
///
/// This now prints:
///
/// ```text
/// Compact: ExampleStruct { name: "Bob", age: DIFF { L: 4, R: 5 } }
/// Pretty: ExampleStruct {
///     name: "Bob",
///     age: DIFF {
///         L: 4,
///         R: 5
///     }
/// }
/// ```
///
/// If you're curious: `debug_diff` uses [`all_different`] to decide to pull a
/// `DIFF` indicator up one level of structure.
///
/// [`all_different`]: fn.all_different.html
pub fn debug_diff<T>(a: T, b: T) -> impl Debug
where
    T: Diff,
{
    DebugDiff(a, b)
}

/// Replacement for the standard `assert_eq!` macro that prints a [`debug_diff`]
/// between its arguments on failure.
///
/// [`debug_diff`]: fn.debug_diff.html
#[macro_export]
macro_rules! assert_eq_diff {
    ($left:expr, $right:expr) => ({
        match (&$left, &$right) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    panic!(r#"assertion failed: `(left == right)`
difference:
{:#?}"#, $crate::debug_diff(left_val, right_val))
                }
            }
        }
    });
    ($left:expr, $right:expr,) => ({
        assert_eq_diff!($left, $right)
    });
    ($left:expr, $right:expr, $($arg:tt)+) => ({
        match (&($left), &($right)) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    panic!(r#"assertion failed: `(left == right)`
{}
difference:
{:#?}"#,
                            format_args!($($arg)+),
                            $crate::debug_diff(left_val, right_val))
                }
            }
        }
    });
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;
    use crate::tests::{TestEnum, TestStruct};

    #[test]
    fn debug_self_usize() {
        let x = 32_usize;
        let formatted = format!("{:?}", DebugDiff(&x, &x));
        assert_eq!(formatted, "32");
    }

    #[test]
    fn debug_self_struct() {
        let a = TestStruct {
            distance: 12,
            silly: false,
        };
        let formatted = format!("{:?}", a);
        let diff = format!("{:?}", DebugDiff(&a, &a));
        assert_eq!(diff, formatted);
    }

    #[test]
    fn debug_delta_struct() {
        let a = TestStruct {
            distance: 12,
            silly: false,
        };
        let b = TestStruct {
            distance: 10,
            silly: false,
        };
        #[rustfmt::skip]
        let expected = "TestStruct { distance: DIFF { L: 12, R: 10 }, \
                                     silly: false }";

        let diff = format!("{:?}", DebugDiff(&a, &b));
        assert_eq!(diff, expected);
    }

    #[test]
    fn debug_full_delta_struct() {
        let a = TestStruct {
            distance: 12,
            silly: false,
        };
        let b = TestStruct {
            distance: 10, // different
            silly: true,  // also different
        };
        #[rustfmt::skip]
        let expected = "DIFF { L: TestStruct { distance: 12, silly: false }, \
                               R: TestStruct { distance: 10, silly: true } }";

        let diff = format!("{:?}", DebugDiff(&a, &b));
        assert_eq!(diff, expected);
    }

    #[test]
    fn debug_delta_struct_pretty() {
        let a = TestStruct {
            distance: 12,
            silly: false,
        };
        let b = TestStruct {
            distance: 10,
            silly: false,
        };

        let expected = "\
TestStruct {
    distance: DIFF {
        L: 12,
        R: 10,
    },
    silly: false,
}";

        let diff = format!("{:#?}", DebugDiff(&a, &b));
        assert_eq!(diff, expected);
    }

    #[test]
    fn debug_enum_same() {
        let diff =
            format!("{:#?}", DebugDiff(&TestEnum::First, &TestEnum::First));
        assert_eq!(diff, format!("{:#?}", TestEnum::First));
    }

    #[test]
    fn debug_enum_different() {
        let diff =
            format!("{:?}", DebugDiff(&TestEnum::First, &TestEnum::Second));
        assert_eq!(diff, "DIFF { L: First, R: Second }");
    }

    #[test]
    fn struct_variant_same() {
        let a = TestEnum::Struct { a: 12, b: false };
        let diff = format!("{:?}", DebugDiff(&a, &a));
        assert_eq!(diff, format!("{:?}", a));
    }

    #[test]
    fn struct_variant_different() {
        let a = TestEnum::Struct { a: 12, b: false };
        let b = TestEnum::Struct { a: 14, b: false };

        let diff = format!("{:?}", DebugDiff(&a, &b));

        assert_eq!(diff, "Struct { a: DIFF { L: 12, R: 14 }, b: false }");
    }

    #[test]
    fn map() {
        use std::collections::BTreeMap;

        let a: BTreeMap<usize, bool> =
            [(0, true), (2, false)].iter().cloned().collect();
        let b: BTreeMap<usize, bool> =
            [(0, false), (1, false)].iter().cloned().collect();

        println!("{:#?}", DebugDiff(&a, &b));
    }
}
