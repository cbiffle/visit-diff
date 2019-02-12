#![allow(unused)]

use diffwalk::Diff;

#[derive(Diff, Debug)]
struct TestStruct {
    a: bool,
    b: (),
}

#[test]
fn foo() {}
