use std::fmt::Debug;
use void::Void;

trait Diff {
    fn diff<'a, D>(a: &'a Self, b: &'a Self, out: D)
        -> Result<D::Ok, D::Err>
        where D: Differ;
}

trait Differ {
    type Ok;
    type Err;

    type StructDiffer: StructDiffer<Ok = Self::Ok, Err = Self::Err>;
    /*
    type StructVariantDiffer: Differ;
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

    /*
    /// Begin traversing a struct variant.
    fn begin_struct_variant(self, ty: &'static str, var: &'static str)
        -> Self::StructVariantDiffer;

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
        -> Result<(), Self::Err>
    where T: Diff;

    fn end(self) -> Result<Self::Ok, Self::Err>;
}


/////////////////////////////////////////////////////////////
// Example differ

#[derive(Copy, Clone, Debug)]
struct Detector;

impl Differ for Detector {
    type Ok = bool;
    type Err = Void;

    type StructDiffer = StructDetector;

    fn difference(self, _: &Debug, _: &Debug) -> Result<Self::Ok, Self::Err> {
        Ok(true)
    }

    fn same(self, _: &Debug, _: &Debug) -> Result<Self::Ok, Self::Err> {
        Ok(false)
    }

    fn diff_newtype<T: ?Sized>(self, _: &'static str, a: &T, b: &T)
        -> Result<Self::Ok, Self::Err>
    where T: Diff
    {
        Diff::diff(a, b, self)
    }

    /// Begin traversing a struct.
    fn begin_struct(self, _: &'static str) -> Self::StructDiffer {
        StructDetector(false)
    }
}

#[derive(Clone, Debug)]
struct StructDetector(bool);

impl StructDiffer for StructDetector {
    type Ok = bool;
    type Err = Void;

    fn diff_field<T: ?Sized>(&mut self, _: &'static str, a: &T, b: &T)
        -> Result<(), Self::Err>
    where T: Diff
    {
        if !self.0 {
            self.0 = Diff::diff(a, b, Detector)?
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Err> {
        Ok(self.0)
    }
}


////////////////////////////////////////////////////////////
// Example printing differ

struct DebugDiff<'a, 'b: 'a>(&'b mut std::fmt::Formatter<'a>);

impl<'a, 'b> Differ for DebugDiff<'a, 'b> {
    type Ok = ();
    type Err = std::fmt::Error;

    type StructDiffer = DebugStructDiff<'a, 'b>;

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

    /// Begin traversing a struct.
    fn begin_struct(self, name: &'static str) -> Self::StructDiffer {
        DebugStructDiff(self.0.debug_struct(name))
    }
}

struct DebugStructDiff<'a, 'b: 'a>(std::fmt::DebugStruct<'a, 'b>);

impl<'a, 'b: 'a> StructDiffer for DebugStructDiff<'a, 'b> {
    type Ok = ();
    type Err = std::fmt::Error;

    fn diff_field<T: ?Sized>(&mut self, name: &'static str, a: &T, b: &T)
        -> Result<(), Self::Err>
    where T: Diff
    {
        self.0.field(name, &FieldDiff(a, b) as &dyn std::fmt::Debug);
        Ok(())
    }

    fn end(mut self) -> Result<Self::Ok, Self::Err> {
        self.0.finish()
    }
}

struct FieldDiff<'a, T: ?Sized>(&'a T, &'a T);

impl<'a, T: ?Sized> std::fmt::Debug for FieldDiff<'a, T>
    where T: Diff
{
    fn fmt<'b, 'c>(&self, fmt: &'c mut std::fmt::Formatter<'b>) -> std::fmt::Result {
        let x = DebugDiff(unsafe { std::mem::transmute(fmt) });
        Diff::diff(self.0, self.1, x)
    }
}


//////////////////////////////////////////////////////////////
// Impls

impl Diff for bool {
    fn diff<'a, D>(a: &'a Self, b: &'a Self, out: D) -> Result<D::Ok, D::Err>
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
    fn diff<'a, D>(a: &'a Self, b: &'a Self, out: D) -> Result<D::Ok, D::Err>
        where D: Differ,
    {
        if a != b {
            out.difference(a, b)
        } else {
            out.same(a, b)
        }
    }
}

#[derive(Clone, Debug)]
struct TestStruct {
    distance: usize,
    silly: bool,
}

impl Diff for TestStruct {
    fn diff<'a, D>(a: &'a Self, b: &'a Self, out: D) -> Result<D::Ok, D::Err>
        where D: Differ,
    {
        let mut s = out.begin_struct("TestStruct");
        s.diff_field("distance", &a.distance, &b.distance)?;
        s.diff_field("silly", &a.silly, &b.silly)?;
        s.end()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use void::ResultVoidExt;

    #[test]
    fn detector_self_false() {
        let a = TestStruct { distance: 12, silly: false };
        assert_eq!(Diff::diff(&a, &a, Detector).void_unwrap(), false)
    }

    #[test]
    fn detector_other_false() {
        let a = TestStruct { distance: 12, silly: false };
        assert_eq!(Diff::diff(&a, &a.clone(), Detector).void_unwrap(), false)
    }

    #[test]
    fn detector_first_field_true() {
        let a = TestStruct { distance: 12, silly: false };
        let b = TestStruct { distance: 10, silly: false };
        assert!(Diff::diff(&a, &b, Detector).void_unwrap())
    }

    #[test]
    fn detector_second_field_true() {
        let a = TestStruct { distance: 12, silly: false };
        let b = TestStruct { distance: 12, silly: true };
        assert!(Diff::diff(&a, &b, Detector).void_unwrap())
    }

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
}
