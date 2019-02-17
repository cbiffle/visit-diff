# `visit_diff`: analyzing structural differences in Rust

[![Build Status](https://travis-ci.org/cbiffle/visit-diff.svg?branch=master)](https://travis-ci.org/cbiffle/visit-diff)

This is a library for easily comparing Rust data structures to detect
differences. This is useful, for example, when reporting a test failure.

The simplest use case:

1. Put a `#[derive(Diff)]` annotation on your type.

2. Replace `assert_eq!` with `assert_eq_diff!`.

Now your error messages will highlight diffs instead of making you hunt for them
manually.

See the API docs on the module for more.
