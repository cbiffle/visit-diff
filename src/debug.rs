use crate::{Diff, Differ, MapDiffer, SeqDiffer, StructDiffer, TupleDiffer};
use std::fmt::Debug;

struct DebugDiff<'a, 'b: 'a>(&'a mut std::fmt::Formatter<'b>);

impl<'a, 'b> Differ for DebugDiff<'a, 'b> {
    type Ok = ();
    type Err = std::fmt::Error;

    type StructDiffer = DebugStructDiff<'a, 'b>;
    type StructVariantDiffer = DebugStructDiff<'a, 'b>;
    type TupleDiffer = DebugTupleDiff<'a, 'b>;
    type TupleVariantDiffer = DebugTupleDiff<'a, 'b>;
    type SeqDiffer = DebugSeqDiff<'a, 'b>;
    type MapDiffer = DebugMapDiff<'a, 'b>;

    fn difference(self, a: &Debug, b: &Debug) -> Result<Self::Ok, Self::Err> {
        self.0
            .debug_struct("DIFF")
            .field("L", a)
            .field("R", b)
            .finish()
    }

    fn same(self, a: &Debug, _: &Debug) -> Result<Self::Ok, Self::Err> {
        a.fmt(self.0)
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

struct DebugStructDiff<'a, 'b>(
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
            f.field(name, &FieldDiff(a, b) as &dyn std::fmt::Debug);
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Err> {
        self.0.and_then(|mut f| f.finish())
    }
}

struct DebugTupleDiff<'a, 'b>(
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
            f.field(&FieldDiff(a, b) as &dyn std::fmt::Debug);
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Err> {
        self.0.and_then(|mut f| f.finish())
    }
}

struct DebugSeqDiff<'a, 'b>(
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
            f.entry(&FieldDiff(a, b) as &dyn std::fmt::Debug);
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Err> {
        self.0.and_then(|mut f| f.finish())
    }
}

struct DebugMapDiff<'a, 'b>(
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
            f.entry(&k, &FieldDiff(a, b) as &dyn std::fmt::Debug);
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

struct FieldDiff<'a, T: ?Sized>(&'a T, &'a T);

impl<'a, T: ?Sized> std::fmt::Debug for FieldDiff<'a, T>
where
    T: Diff,
{
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        Diff::diff(self.0, self.1, DebugDiff(fmt))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{TestEnum, TestStruct};

    #[test]
    fn debug_self_usize() {
        let x = 32_usize;
        let formatted = format!("{:?}", FieldDiff(&x, &x));
        assert_eq!(formatted, "32");
    }

    #[test]
    fn debug_self_struct() {
        let a = TestStruct {
            distance: 12,
            silly: false,
        };
        let formatted = format!("{:?}", a);
        let diff = format!("{:?}", FieldDiff(&a, &a));
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
        let expected = "TestStruct { distance: DIFF { L: 12, R: 10 }, \
                        silly: false }";

        let diff = format!("{:?}", FieldDiff(&a, &b));
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

        let diff = format!("{:#?}", FieldDiff(&a, &b));
        assert_eq!(diff, expected);
    }

    #[test]
    fn debug_enum_same() {
        let diff =
            format!("{:#?}", FieldDiff(&TestEnum::First, &TestEnum::First));
        assert_eq!(diff, format!("{:#?}", TestEnum::First));
    }

    #[test]
    fn debug_enum_different() {
        let diff =
            format!("{:?}", FieldDiff(&TestEnum::First, &TestEnum::Second));
        assert_eq!(diff, "DIFF { L: First, R: Second }");
    }

    #[test]
    fn struct_variant_same() {
        let a = TestEnum::Struct { a: 12, b: false };
        let diff = format!("{:?}", FieldDiff(&a, &a));
        assert_eq!(diff, format!("{:?}", a));
    }

    #[test]
    fn struct_variant_different() {
        let a = TestEnum::Struct { a: 12, b: false };
        let b = TestEnum::Struct { a: 14, b: true };

        let diff = format!("{:?}", FieldDiff(&a, &b));

        assert_eq!(
            diff,
            "Struct { a: DIFF { L: 12, R: 14 }, b: DIFF { L: false, R: true } }"
        );
    }

    #[test]
    fn map() {
        use std::collections::BTreeMap;

        let a: BTreeMap<usize, bool> =
            [(0, true), (2, false)].iter().cloned().collect();
        let b: BTreeMap<usize, bool> =
            [(0, false), (1, false)].iter().cloned().collect();

        println!("{:#?}", FieldDiff(&a, &b));
    }
}
