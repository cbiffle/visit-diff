//! Report differences using `Debug` and `Formatter`.

use crate::{
    Diff, Differ, MapDiffer, SeqDiffer, SetDiffer, StructDiffer, TupleDiffer,
};
use std::fmt::Debug;

use super::detect::all_different;

/// Adapts a `std::fmt::Formatter` into a `Differ`.
pub struct DebugDiffer<'a, 'b>(&'a mut std::fmt::Formatter<'b>);

impl<'a, 'b> Differ for DebugDiffer<'a, 'b> {
    type Ok = ();
    type Err = std::fmt::Error;

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

impl std::fmt::Debug for Missing {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.write_str("(missing)")
    }
}

pub struct DebugStructDiff<'a, 'b>(
    Result<std::fmt::DebugStruct<'a, 'b>, std::fmt::Error>,
);

impl<'a, 'b> StructDiffer for DebugStructDiff<'a, 'b> {
    type Ok = ();
    type Err = std::fmt::Error;

    fn diff_field<T: ?Sized>(&mut self, name: &'static str, a: &T, b: &T)
    where
        T: Diff,
    {
        if let Ok(f) = &mut self.0 {
            f.field(name, &DebugDiff(a, b) as &dyn std::fmt::Debug);
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Err> {
        self.0.and_then(|mut f| f.finish())
    }
}

pub struct DebugTupleDiff<'a, 'b>(
    Result<std::fmt::DebugTuple<'a, 'b>, std::fmt::Error>,
);

impl<'a, 'b> TupleDiffer for DebugTupleDiff<'a, 'b> {
    type Ok = ();
    type Err = std::fmt::Error;

    fn diff_field<T: ?Sized>(&mut self, a: &T, b: &T)
    where
        T: Diff,
    {
        if let Ok(f) = &mut self.0 {
            f.field(&DebugDiff(a, b) as &dyn std::fmt::Debug);
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Err> {
        self.0.and_then(|mut f| f.finish())
    }
}

pub struct DebugSeqDiff<'a, 'b>(
    Result<std::fmt::DebugList<'a, 'b>, std::fmt::Error>,
);

impl<'a, 'b> SeqDiffer for DebugSeqDiff<'a, 'b> {
    type Ok = ();
    type Err = std::fmt::Error;

    fn diff_element<T: ?Sized>(&mut self, a: &T, b: &T)
    where
        T: Diff,
    {
        if let Ok(f) = &mut self.0 {
            f.entry(&DebugDiff(a, b) as &dyn std::fmt::Debug);
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

pub struct DebugSetDiff<'a, 'b>(
    Result<std::fmt::DebugSet<'a, 'b>, std::fmt::Error>,
);

impl<'a, 'b> SetDiffer for DebugSetDiff<'a, 'b> {
    type Ok = ();
    type Err = std::fmt::Error;

    fn diff_equal<V>(&mut self, a: &V, b: &V)
    where
        V: ?Sized + Diff,
    {
        if let Ok(f) = &mut self.0 {
            f.entry(&DebugDiff(a, b) as &dyn std::fmt::Debug);
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

pub struct DebugMapDiff<'a, 'b>(
    Result<std::fmt::DebugMap<'a, 'b>, std::fmt::Error>,
);

impl<'a, 'b> MapDiffer for DebugMapDiff<'a, 'b> {
    type Ok = ();
    type Err = std::fmt::Error;

    fn diff_entry<K, V>(&mut self, k: &K, a: &V, b: &V)
    where
        K: ?Sized + Debug,
        V: ?Sized + Diff,
    {
        if let Ok(f) = &mut self.0 {
            f.entry(&k, &DebugDiff(a, b) as &dyn std::fmt::Debug);
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
pub struct DebugDiff<'a, T: ?Sized>(pub &'a T, pub &'a T);

impl<'a, T: ?Sized> std::fmt::Debug for DebugDiff<'a, T>
where
    T: Diff,
{
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        if all_different(self.0, self.1) {
            DebugDiffer(fmt).difference(&self.0, &self.1)
        } else {
            Diff::diff(self.0, self.1, DebugDiffer(fmt))
        }
    }
}

#[cfg(test)]
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
        R: 10
    },
    silly: false
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
