use visit_diff::debug_diff;
use visit_diff::Diff;

#[macro_use]
mod common;

/// enum variations
#[derive(Copy, Clone, Diff, Debug)]
enum TestEnum {
    /// unit
    A,
    /// named fields
    B { unit: (), size: usize },
    /// unnamed fields
    C(bool, usize),
}

/// zero-variant enum
#[derive(Diff, Debug)]
#[allow(unused)] // just making sure it compiles
enum EnumZ {}

debug_equivalence! {
    unit => TestEnum::A;
    r#struct => TestEnum::B { unit: (), size: 12 };
    tuple => TestEnum::C(true, 42);
}
