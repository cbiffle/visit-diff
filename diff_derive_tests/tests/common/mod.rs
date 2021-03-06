#[macro_export]
macro_rules! debug_equivalence {
    ($($name:ident => $x:expr;)*) => {
        $(
            #[test]
            fn $name() {
                let x = $x;
                assert_eq!(format!("{:?}", x),
                format!("{:?}", debug_diff(&x, &x)));
            }
        )*
    };
}
