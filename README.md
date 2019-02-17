# `visit_diff`: analyzing structural differences in Rust

This is a library for easily comparing Rust data structures to detect
differences. This is useful, for example, when reporting a test failure.

The mechanism uses two cooperating traits:

- `Diff` is implemented by a structure that can be compared.
- `Differ` is implemented by code that does something with the results.

This lets you replace the code responding to the diff (the `Differ`) with the
strategy of your choice. Because the interaction between the traits is all
static, the two are optimized together at no runtime cost.

(If you've used `serde` this will look familiar.)
