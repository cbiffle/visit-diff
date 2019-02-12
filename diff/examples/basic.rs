//! Uses the `diffwalk::debug` module to print differences between two data
//! structures in `Debug` format.

use diffwalk::Diff;
use diffwalk::debug::DebugDiff;

////////////////////////////////////////////////////////////////////////////////
// Arbitrary example data structure.

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
    let a = Top {
        child1: Child1 { name: "Sprocket", size: 12 },
        others: vec![
            Other::Prince,
            Other::Bob { last_name: "Roberts" },
        ],
    };

    let b = Top {
        // Note: both name and size are different.
        child1: Child1 { name: "Ralph", size: usize::max_value() },
        others: vec![
            Other::Prince,
            Other::Bob { last_name: "Roberts" },
            Other::Bob { last_name: "Bobberson" }, // added
        ],
    };

    println!("{:#?}", DebugDiff(&a, &b));
}
