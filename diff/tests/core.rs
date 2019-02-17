#[macro_use]
mod common;

debug_equivalence! {
    actual_unit => ();
    refs => &&mut &();
    bool => true;
    u32 => 42u32;
    str => "hello, world";
    tuple => (true, 42u32, ());
    array => [0u32, 1, 2, 3];
    slice => &[0u32, 1, 2, 3] as &[u32];
    cell => core::cell::Cell::new(42u32);
    ref_cell => core::cell::RefCell::new(42u32);
}
