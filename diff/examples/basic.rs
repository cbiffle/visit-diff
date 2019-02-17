//! Uses the `visit_diff::debug` module to print differences between two data
//! structures in `Debug` format.

use visit_diff::debug_diff;
use visit_diff::Diff;

////////////////////////////////////////////////////////////////////////////////
// Arbitrary example data structure.

#[derive(Debug, Diff)]
struct Newtype(Top);

#[derive(Debug, Diff)]
struct Top {
    pub child1: Child1,
    pub others: Vec<Other>,
}

#[derive(Debug, Diff)]
struct Child1 {
    pub name: &'static str,
    pub size: usize,
}

#[derive(Debug, Diff)]
enum Other {
    Prince,
    Bob { last_name: &'static str },
}

////////////////////////////////////////////////////////////////////////////////
// Actual code.

fn main() {
    let a = Newtype(Top {
        child1: Child1 {
            name: "Sprocket",
            size: 12,
        },
        others: vec![
            Other::Prince,
            Other::Bob {
                last_name: "Roberts",
            },
        ],
    });

    let b = Newtype(Top {
        // Note: both name and size are different.
        child1: Child1 {
            name: "Ralph",
            size: usize::max_value(),
        },
        others: vec![
            Other::Prince,
            Other::Bob {
                last_name: "Roberts",
            },
            // added
            Other::Bob {
                last_name: "Bobberson",
            },
        ],
    });

    println!("{:#?}", debug_diff(a, b));
}
