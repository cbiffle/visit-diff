use std::fmt::Debug;
use crate::{Diff, Differ, StructDiffer};

struct DebugDiff<'a, 'b: 'a>(&'a mut std::fmt::Formatter<'b>);

impl<'a, 'b> Differ for DebugDiff<'a, 'b> {
    type Ok = ();
    type Err = std::fmt::Error;

    type StructDiffer = DebugStructDiff<'a, 'b>;
    type StructVariantDiffer = DebugStructDiff<'a, 'b>;

    fn difference(self, a: &Debug, b: &Debug) -> Result<Self::Ok, Self::Err> {
        self.0.debug_struct("DIFF")
            .field("L", a)
            .field("R", b)
            .finish()
    }

    fn same(self, a: &Debug, _: &Debug) -> Result<Self::Ok, Self::Err> {
        a.fmt(self.0)
    }

    fn diff_newtype<T: ?Sized>(self, _: &'static str, a: &T, b: &T)
        -> Result<Self::Ok, Self::Err>
    where T: Diff
    {
        Diff::diff(a, b, self)
    }

    fn begin_struct(self, name: &'static str) -> Self::StructDiffer {
        DebugStructDiff(Ok(self.0.debug_struct(name)))
    }

    fn begin_struct_variant(self, ty: &'static str, v: &'static str)
        -> Self::StructVariantDiffer
    {
        let DebugDiff(fmt) = self;
        let result = fmt.write_str(ty)
            .and_then(|_| fmt.write_str("::"))
            .map(move |_| fmt.debug_struct(v));
        DebugStructDiff(result)
    }
}

struct DebugStructDiff<'a, 'b>(
    Result<std::fmt::DebugStruct<'a, 'b>, std::fmt::Error>
);

impl<'a, 'b> StructDiffer for DebugStructDiff<'a, 'b> {
    type Ok = ();
    type Err = std::fmt::Error;

    fn diff_field<T: ?Sized>(&mut self, name: &'static str, a: &T, b: &T)
    where T: Diff
    {
        if let Ok(f) = &mut self.0 {
            f.field(name, &FieldDiff(a, b) as &dyn std::fmt::Debug);
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Err> {
        self.0.and_then(|mut f| f.finish())
    }
}

struct FieldDiff<'a, T: ?Sized>(&'a T, &'a T);

impl<'a, T: ?Sized> std::fmt::Debug for FieldDiff<'a, T>
    where T: Diff
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
        let a = TestStruct { distance: 12, silly: false };
        let formatted = format!("{:?}", a);
        let diff = format!("{:?}", FieldDiff(&a, &a));
        assert_eq!(diff, formatted);
    }

    #[test]
    fn debug_delta_struct() {
        let a = TestStruct { distance: 12, silly: false };
        let b = TestStruct { distance: 10, silly: false };
        let expected =
            "TestStruct { distance: DIFF { L: 12, R: 10 }, \
                          silly: false }";

        let diff = format!("{:?}", FieldDiff(&a, &b));
        assert_eq!(diff, expected);
    }

    #[test]
    fn debug_delta_struct_pretty() {
        let a = TestStruct { distance: 12, silly: false };
        let b = TestStruct { distance: 10, silly: false };

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
        let diff = format!("{:#?}", FieldDiff(&TestEnum::First, &TestEnum::First));
        assert_eq!(diff, format!("{:#?}", TestEnum::First));
    }

    #[test]
    fn debug_enum_different() {
        let diff = format!("{:?}", FieldDiff(&TestEnum::First, &TestEnum::Second));
        assert_eq!(diff, "DIFF { L: First, R: Second }");
    }

}
