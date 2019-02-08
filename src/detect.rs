use std::fmt::Debug;
use void::{Void, ResultVoidExt};

use crate::{Differ, Diff, StructDiffer};

#[derive(Copy, Clone, Debug)]
struct Detector;

impl Differ for Detector {
    type Ok = bool;
    type Err = Void;

    type StructDiffer = StructDetector;
    type StructVariantDiffer = StructDetector;

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

    fn begin_struct_variant(self, _: &'static str, _: &'static str)
        -> Self::StructVariantDiffer
    {
        StructDetector(false)
    }
}

#[derive(Clone, Debug)]
struct StructDetector(bool);

impl StructDiffer for StructDetector {
    type Ok = bool;
    type Err = Void;

    fn diff_field<T: ?Sized>(&mut self, _: &'static str, a: &T, b: &T)
    where T: Diff
    {
        if !self.0 {
            self.0 = Diff::diff(a, b, Detector).void_unwrap();
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Err> {
        Ok(self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{TestStruct, TestEnum};
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
    fn detector_enum() {
        assert_eq!(Diff::diff(&TestEnum::First, &TestEnum::First, Detector)
                   .void_unwrap(), false);
        assert_eq!(Diff::diff(&TestEnum::Second, &TestEnum::Second, Detector)
                   .void_unwrap(), false);

        assert!(Diff::diff(&TestEnum::First, &TestEnum::Second, Detector)
                .void_unwrap());
        assert!(Diff::diff(&TestEnum::Second, &TestEnum::First, Detector)
                .void_unwrap());
    }
}
