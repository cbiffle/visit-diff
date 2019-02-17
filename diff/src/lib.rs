//! Analyzing structural differences in Rust values using visitors.
//!
//! # Simple application
//!
//! This crate provides three functions that you can use immediately without
//! having to learn a bunch of traits.
//!
//! - [`debug_diff`] enables you to print the differences between two values of
//!   a [`Diff`] type using debug formatting.
//!
//! - [`any_difference`] and [`all_different`] scan values for differences and
//!   return a `bool`.
//!
//! You can derive [`Diff`] for any custom type that implements `Debug`.
//!
//! # Under the hood
//!
//! This scheme is modeled after a combination of `core::fmt::Formatter` and
//! `serde::Serialize`. There are two main traits:
//!
//! - [`Diff`] is implemented by types that can be diff'd.
//! - [`Differ`] is implemented by types that can process differences.
//!
//! You'll typically derive [`Diff`]. Derived impls will simply present the
//! structure of the type honestly, much like a derived `Debug` impl would.
//! However, you can also implement it by hand if you need special
//! functionality.
//!
//! The most detailed docs are on the [`Differ`] trait.
//!
//! # Visitors
//!
//! Together, [`Diff`] and [`Differ`] implement the [Visitor Pattern] for
//! climbing over a data structure. This is a little different than some other
//! applications of visitors you might have encountered. In particular:
//!
//! 1. The structure being visited is *the Rust notion of types*: here is a
//!    struct, the struct has fields, etc. A custom impl of [`Diff`] can fake
//!    the internals of a type to abstract away details, but the model is still
//!    the same.
//!
//! 2. Instead of visiting the parts of a *single* data structure, here we are
//!    visiting *two* data structures of the same type in parallel. This means
//!    we stop visiting if the structures diverge -- for example, if we discover
//!    two *different* variants of an `enum` type. (When this happens we notify
//!    the [`Differ`] through the [`difference`] method.)
//!
//! 3. The [double dispatch] aspect of the visitor pattern occurs *at compile
//!    time,* rather than at runtime, so there's very little overhead.
//!    The description of the pattern on Wikipedia (and the book *Design
//!    Patterns* that originated the name "visitor") doesn't discuss this
//!    version, only considering `dyn`-style runtime dispatch.
//!
//! # `no_std` support
//!
//! This crate is `no_std` compatible, in case you want to diff data structures
//! in a deeply-embedded system.
//!
//! [`Diff`]: trait.Diff.html
//! [`Differ`]: trait.Differ.html
//! [`any_difference`]: fn.any_difference.html
//! [`all_different`]: fn.all_different.html
//! [`debug_diff`]: fn.debug_diff.html
//! [Visitor Pattern]: https://en.wikipedia.org/wiki/Visitor_pattern
//! [double dispatch]: https://en.wikipedia.org/wiki/Double_dispatch
//! [`difference`]: trait.Differ.html#tymethod.difference

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "visit_diff_derive")]
pub use visit_diff_derive::*;

mod debug;
mod detect;
mod unit;
#[macro_use]
mod impls;
#[cfg(feature = "std")]
mod std_impls;

use core::fmt::Debug;
use itertools::{EitherOrBoth, Itertools};

pub use debug::debug_diff;
pub use detect::{all_different, any_difference};

/// A type that can be compared structurally to discover differences.
///
/// The job of a `Diff` impl is to use knowledge of the structure of some type
/// (`Self`) to check two copies for differences, and report findings to a
/// [`Differ`].
///
/// [`Differ`] has methods for each different flavor of Rust type, and you can
/// only call one of them (as they consume the `Differ`). So, the first task
/// when implementing `Diff` is to decide which one to call.
///
/// Very simple types may just dispatch to [`same`] or [`difference`], like this
/// impl for a newtype around `u32`:
///
/// ```
/// use visit_diff::{Diff, Differ};
///
/// #[derive(Debug)]
/// struct MyInt(u32);
///
/// impl Diff for MyInt {
///     fn diff<D>(a: &Self, b: &Self, out: D) -> Result<D::Ok, D::Err>
///     where D: Differ
///     {
///         if a.0 == b.0 {
///             out.same(&a, &b)
///         } else {
///             out.difference(&a, &b)
///         }
///     }
/// }
///
/// use visit_diff::any_difference;
///
/// assert_eq!(any_difference(&MyInt(1), &MyInt(1)), false);
/// assert_eq!(any_difference(&MyInt(1), &MyInt(0)), true);
/// ```
///
/// (Note: in reality, you'd probably want to `#[derive(Diff)]` for a type this
/// simple.)
///
/// More complicated types would use other methods on [`Differ`] to describe
/// their structure. See the documentation of that trait for more.
///
/// [`Differ`]: trait.Differ.html
/// [`same`]: trait.Differ.html#tymethod.same
/// [`difference`]: trait.Differ.html#tymethod.difference
pub trait Diff: Debug {
    /// Inspect `a` and `b` and tell `out` about any differences.
    ///
    /// All (reasonable) implementations of this method have the same basic
    /// structure: they call a method on `out`, and then follow the instructions
    /// from that method to get a result.
    fn diff<D>(a: &Self, b: &Self, out: D) -> Result<D::Ok, D::Err>
    where
        D: Differ;
}

/// A type that can do something with information about structural differences.
///
/// If you think that sounds vague, you're right! This trait is very general and
/// covers a lot of use cases, which can make it hard to understand at first
/// glance. Don't worry! It's straightforward once you get the hang of it,
/// though it *is* pretty wordy.
///
/// # How to use a `Differ`
///
/// Normally, you'll only call the methods on `Differ` from within an
/// implementation of [`Diff::diff`]. Also, normally, you won't write your own
/// implementation of [`Diff::diff`] in the first place -- you'll
/// `#[derive(Diff)]`. This section will explain how to use `Differ` manually to
/// produce the same results as the derived impls, should you ever need to.
///
/// An implementation of `Differ` is actually a small *family* of types. There's
/// the type implementing `Differ` (which we'll call "the differ") for short,
/// and then there are the *associated types*. All of the methods on `Differ`
/// either produce a result immediately, or convert the differ into one of the
/// associated types because more information is needed to produce a result.
///
/// In the end, every complete interaction with a differ type `D` produces the
/// same type: `Result<D::Ok, D::Err>`. This means each implementation can
/// decide what its output and failure types look like.
///
/// The basic methods [`difference`], [`same`], and [`diff_newtype`] produce a
/// result immediately without further work.
///
/// The methods starting with `begin` require more than one step.
///
/// ## `struct`
///
/// If a type is a struct with named fields, call [`begin_struct`] to convert
/// the `Differ` into a [`StructDiffer`]. `StructDiffer` has methods for
/// describing struct fields. See the example on [`begin_struct`] for more.
///
/// Not all structs have named fields: there are also tuple structs. For a tuple
/// struct, call [`begin_tuple`] to convert the `Differ` into a [`TupleDiffer`].
/// `TupleDiffer` has methods for describing tuple struct fields. See the
/// example on [`begin_tuple`] for more.
///
/// ## `enum`
///
/// Rust enums are more complex than structs, because each variant of an enum
/// can have a *different shape*: some may have named fields, some may have
/// unnamed fields, and some may be *unit variants* without fields at all.
///
/// Typically you only want to treat two values of an enum type as "same" if
/// they have the same variant. This means an implementation of `diff` for an
/// enum will usually have a "parallel match" shape like this:
///
/// ```
/// use visit_diff::{Differ, Diff};
///
/// #[derive(Debug)]
/// enum ExampleEnum { Variant1, Variant2 }
///
/// impl Diff for ExampleEnum {
///     fn diff<D>(a: &Self, b: &Self, out: D) -> Result<D::Ok, D::Err>
///     where D: Differ
///     {
///         match (a, b) {
///             (ExampleEnum::Variant1, ExampleEnum::Variant1) => {
///                 out.same(a, b)
///             }
///             (ExampleEnum::Variant2, ExampleEnum::Variant2) => {
///                 out.same(a, b)
///             }
///             _ => out.difference(a, b),
///         }
///     }
/// }
/// ```
///
/// In that example, both variants are *unit variants* without fields. Let's
/// consider the other flavors.
///
/// For struct variants with named fields, use [`begin_struct_variant`] to
/// convert the differ into a [`StructVariantDiffer`]. See the example on
/// [`begin_struct_variant`] for more.
///
/// For tuple variants with unnamed fields, use [`begin_tuple_variant`] to
/// convert the differ into a [`TupleVariantDiffer`]. See the example on
/// [`begin_tuple_variant`] for more.
///
/// ## Abstract types
///
/// This crate recognizes three kinds of *abstract types*, which don't directly
/// map to any native Rust type, but are really common library types. (They also
/// happen to be the three kinds of abstract types recognized by
/// `std::fmt::Formatter`.)
///
/// *Sequences* are variable-length ordered collections of things, such as a
/// slice or a `Vec`. Not only can the individual elements be different between
/// two sequences, but elements can be added or removed, too. For types that
/// want to be treated like sequences, use [`begin_seq`] to convert the differ
/// into a [`SeqDiffer`]. See the example on [`begin_seq`] for more.
///
/// *Sets* are variable-length collections of things where each thing appears
/// only once, such as a `HashSet`. Sets may or may not be ordered. They're
/// otherwise treated a lot like sequences. For set-like types, use
/// [`begin_set`] to convert the differ into a [`SetDiffer`].
///
/// *Maps* are variable-length collections of key-value pairs, like a `HashMap`.
/// Maps may or may not be ordered. For map-like types, use [`begin_map`] to
/// convert the differ into a [`MapDiffer`].
///
/// [`Diff::diff`]: trait.Diff.html#tymethod.diff
/// [`difference`]: #tymethod.difference
/// [`same`]: #tymethod.same
/// [`diff_newtype`]: #tymethod.diff_newtype
/// [`begin_struct`]: #tymethod.begin_struct
/// [`begin_struct_variant`]: #tymethod.begin_struct_variant
/// [`begin_tuple`]: #tymethod.begin_tuple
/// [`begin_tuple_variant`]: #tymethod.begin_tuple_variant
/// [`begin_seq`]: #tymethod.begin_seq
/// [`begin_set`]: #tymethod.begin_set
/// [`begin_map`]: #tymethod.begin_map
/// [`StructDiffer`]: trait.StructDiffer.html
/// [`StructVariantDiffer`]: trait.StructVariantDiffer.html
/// [`TupleDiffer`]: trait.TupleDiffer.html
/// [`TupleVariantDiffer`]: trait.TupleVariantDiffer.html
/// [`SeqDiffer`]: trait.SeqDiffer.html
/// [`SetDiffer`]: trait.SetDiffer.html
/// [`MapDiffer`]: trait.MapDiffer.html
pub trait Differ {
    /// Type returned on success.
    type Ok;
    /// Type returned on failure.
    ///
    /// If your differ can't fail, consider using the [`void`] crate. It
    /// provides an extension method on `Result`, `unwrap_void`, that never
    /// panics.
    ///
    /// [`void`]: http://docs.rs/void/
    type Err;

    /// The type we turn into when diffing a struct.
    type StructDiffer: StructDiffer<Ok = Self::Ok, Err = Self::Err>;
    /// The type we turn into when diffing a struct variant of an enum.
    ///
    /// This is often the same type as `StructDiffer`.
    type StructVariantDiffer: StructDiffer<Ok = Self::Ok, Err = Self::Err>;
    /// The type we turn into when diffing a tuple or tuple struct.
    type TupleDiffer: TupleDiffer<Ok = Self::Ok, Err = Self::Err>;
    /// The type we turn into when diffing a tuple variant of an enum.
    ///
    /// This is often the same type as `TupleDiffer`.
    type TupleVariantDiffer: TupleDiffer<Ok = Self::Ok, Err = Self::Err>;
    /// The type we turn into when diffing an abstract sequence.
    type SeqDiffer: SeqDiffer<Ok = Self::Ok, Err = Self::Err>;
    /// The type we turn into when diffing an abstract map.
    type MapDiffer: MapDiffer<Ok = Self::Ok, Err = Self::Err>;
    /// The type we turn into when diffing an abstract set.
    type SetDiffer: SetDiffer<Ok = Self::Ok, Err = Self::Err>;

    /// Two atomic values have been discovered to be different, such as
    /// different numbers or different variants of an enum.
    fn difference(self, a: &Debug, b: &Debug) -> Result<Self::Ok, Self::Err>;

    /// Two atomic values are the same, such as equal numbers or identical unit
    /// variants of an enum.
    fn same(self, a: &Debug, b: &Debug) -> Result<Self::Ok, Self::Err>;

    /// Encounter a newtype. `a` and `b` are the contents of the sole fields of
    /// the left-hand and right-hand value, respectively.
    fn diff_newtype<T: ?Sized>(
        self,
        ty: &'static str,
        a: &T,
        b: &T,
    ) -> Result<Self::Ok, Self::Err>
    where
        T: Diff;

    /// Begin traversing a struct with named fields.
    ///
    /// This converts `self` into an implementation of [`StructDiffer`], which
    /// in turn has methods for describing a struct.
    ///
    /// Here's an example of using `begin_struct` to manually implement [`Diff`]
    /// for a struct with named fields:
    ///
    /// ```
    /// use visit_diff::{Diff, Differ};
    ///
    /// #[derive(Debug)]
    /// struct ExampleStruct {
    ///     name: String,
    ///     age: usize,
    /// }
    ///
    /// impl Diff for ExampleStruct {
    ///     fn diff<D>(a: &Self, b: &Self, out: D) -> Result<D::Ok, D::Err>
    ///     where D: Differ
    ///     {
    ///         // Bring the struct operations into scope. This could also go at
    ///         // the top.
    ///         use visit_diff::StructDiffer;
    ///
    ///         let mut out = out.begin_struct("ExampleStruct");
    ///
    ///         // Visit each field in turn.
    ///         out.diff_field("name", &a.name, &b.name);
    ///         out.diff_field("age", &a.age, &b.age);
    ///
    ///         // Finish the diff and generate the result.
    ///         out.end()
    ///     }
    /// }
    /// ```
    ///
    /// [`StructDiffer`]: trait.StructDiffer.html
    /// [`Diff`]: trait.Diff.html
    fn begin_struct(self, ty: &'static str) -> Self::StructDiffer;

    /// Begin traversing a struct variant of an enum.
    ///
    /// The rest is very similar to dealing with a normal struct, except that we
    /// have to use pattern matching to get at the fields.
    ///
    /// ```
    /// use visit_diff::{Diff, Differ};
    ///
    /// #[derive(Debug)]
    /// enum ExampleEnum {
    ///     Unit,
    ///     Struct {
    ///         name: String,
    ///         age: usize,
    ///     },
    /// }
    ///
    /// impl Diff for ExampleEnum {
    ///     fn diff<D>(a: &Self, b: &Self, out: D) -> Result<D::Ok, D::Err>
    ///     where D: Differ
    ///     {
    ///         match (a, b) {
    ///             (ExampleEnum::Unit, ExampleEnum::Unit) => out.same(a, b),
    ///             (ExampleEnum::Struct { name: a_name, age: a_age },
    ///              ExampleEnum::Struct { name: b_name, age: b_age }) => {
    ///                 // Bring the struct operations into scope. This could
    ///                 // also go at the top. Note that struct variants use the
    ///                 // same trait as normal structs.
    ///                 use visit_diff::StructDiffer;
    ///
    ///                 let mut out = out.begin_struct_variant(
    ///                     "ExampleEnum", // type name
    ///                     "Struct",      // variant name
    ///                 );
    ///
    ///                 // Visit each field in turn.
    ///                 out.diff_field("name", a_name, b_name);
    ///                 out.diff_field("age", a_age, b_age);
    ///
    ///                 // Finish the diff and generate the result.
    ///                 out.end()
    ///             }
    ///             _ => out.difference(a, b),
    ///         }
    ///     }
    /// }
    /// ```
    fn begin_struct_variant(
        self,
        ty: &'static str,
        var: &'static str,
    ) -> Self::StructVariantDiffer;

    /// Begin traversing a tuple struct or raw tuple.
    ///
    /// This converts `self` into an implementation of [`TupleDiffer`], which
    /// in turn has methods for describing a tuple struct.
    ///
    /// To describe something as a raw tuple (even if it isn't necessarily),
    /// pass an empty string for the type name.
    ///
    /// Here's an example of using `begin_tuple` to manually implement [`Diff`]
    /// for a struct with unnamed fields:
    ///
    /// ```
    /// use visit_diff::{Diff, Differ};
    ///
    /// #[derive(Debug)]
    /// struct ExampleStruct(String, usize);
    ///
    /// impl Diff for ExampleStruct {
    ///     fn diff<D>(a: &Self, b: &Self, out: D) -> Result<D::Ok, D::Err>
    ///     where D: Differ
    ///     {
    ///         // Bring the tuple operations into scope. This could also go at
    ///         // the top.
    ///         use visit_diff::TupleDiffer;
    ///
    ///         let mut out = out.begin_tuple("ExampleStruct");
    ///
    ///         // Visit each field in turn.
    ///         out.diff_field(&a.0, &b.0);
    ///         out.diff_field(&a.1, &b.1);
    ///
    ///         // Finish the diff and generate the result.
    ///         out.end()
    ///     }
    /// }
    /// ```
    ///
    /// [`TupleDiffer`]: trait.TupleDiffer.html
    /// [`Diff`]: trait.Diff.html
    fn begin_tuple(self, ty: &'static str) -> Self::TupleDiffer;

    /// Begin traversing a tuple variant of an enum.
    ///
    /// The rest is very similar to dealing with a normal tuple, except that we
    /// have to use pattern matching to get at the fields.
    ///
    /// ```
    /// use visit_diff::{Diff, Differ};
    ///
    /// #[derive(Debug)]
    /// enum ExampleEnum {
    ///     Unit,
    ///     Tuple(String, usize),
    /// }
    ///
    /// impl Diff for ExampleEnum {
    ///     fn diff<D>(a: &Self, b: &Self, out: D) -> Result<D::Ok, D::Err>
    ///     where D: Differ
    ///     {
    ///         match (a, b) {
    ///             (ExampleEnum::Unit, ExampleEnum::Unit) => out.same(a, b),
    ///             (ExampleEnum::Tuple(a_name, a_age),
    ///              ExampleEnum::Tuple(b_name, b_age)) => {
    ///                 // Bring the tuple operations into scope. This could
    ///                 // also go at the top. Note that tuple variants use the
    ///                 // same trait as normal tuples.
    ///                 use visit_diff::TupleDiffer;
    ///
    ///                 let mut out = out.begin_tuple_variant(
    ///                     "ExampleEnum", // type name
    ///                     "Tuple",      // variant name
    ///                 );
    ///
    ///                 // Visit each field in turn.
    ///                 out.diff_field(a_name, b_name);
    ///                 out.diff_field(a_age, b_age);
    ///
    ///                 // Finish the diff and generate the result.
    ///                 out.end()
    ///             }
    ///             _ => out.difference(a, b),
    ///         }
    ///     }
    /// }
    /// ```
    fn begin_tuple_variant(
        self,
        ty: &'static str,
        var: &'static str,
    ) -> Self::TupleVariantDiffer;

    /// Begin traversing a sequence.
    ///
    /// This is quite general; it's up to you to decide how exactly your type
    /// looks like a sequence.
    ///
    /// Here's a simple implementation for slices -- which we wrap in a newtype
    /// here because there's already an implementation for slices. This uses the
    /// provided [`diff_elements`] method that makes diffing two iterators easy.
    ///
    /// ```
    /// use visit_diff::{Diff, Differ};
    ///
    /// #[derive(Debug)]
    /// struct Slice<'a, T>(&'a [T]);
    ///
    /// impl<'a, T: Diff> Diff for Slice<'a, T> {
    ///     fn diff<D>(a: &Self, b: &Self, out: D) -> Result<D::Ok, D::Err>
    ///     where D: Differ
    ///     {
    ///         // Bring the sequence operations into scope. This could also go
    ///         // at the top.
    ///         use visit_diff::SeqDiffer;
    ///
    ///         let mut out = out.begin_seq();
    ///         out.diff_elements(a.0, b.0);
    ///         out.end()
    ///     }
    /// }
    /// ```
    ///
    /// [`diff_elements`]: trait.SeqDiffer.html#method.diff_elements
    fn begin_seq(self) -> Self::SeqDiffer;

    /// Begin traversing a map.
    fn begin_map(self) -> Self::MapDiffer;

    /// Begin traversing a set.
    fn begin_set(self) -> Self::SetDiffer;
}

/// A type that can deal with differences in a `struct`.
pub trait StructDiffer {
    /// Type returned on success.
    type Ok;
    /// Type returned on failure.
    type Err;

    /// Visits a field `name` with values `a` and `b` in the respective
    /// structures.
    fn diff_field<T: ?Sized>(&mut self, name: &'static str, a: &T, b: &T)
    where
        T: Diff;

    /// Skips a field that is excluded from differencing.
    ///
    /// Some differs may e.g. print a placeholder for skipped fields.
    fn skip_field<T: ?Sized>(&mut self, _name: &'static str) {}

    /// Completes traversal of the struct.
    fn end(self) -> Result<Self::Ok, Self::Err>;
}

/// A type that can do something with information about differences in a
/// tuple or tuple-like struct.
///
/// For two tuples to be of the same type (and thus be able to be diffed), they
/// must be the same length. For types that vary in length, you want a
/// [`SeqDiffer`] instead.
///
/// [`SeqDiffer`]: trait.SeqDiffer.html
pub trait TupleDiffer {
    /// Type returned on success.
    type Ok;
    /// Type returned on failure.
    type Err;

    /// Visits the *next* field in each tuple. The field number is implicit.
    fn diff_field<T: ?Sized>(&mut self, a: &T, b: &T)
    where
        T: Diff;

    /// Signals that a field is being skipped. Some differs may do something
    /// with this information.
    fn skip_field<T: ?Sized>(&mut self) {}

    /// Finish diffing the tuples and return a result.
    fn end(self) -> Result<Self::Ok, Self::Err>;
}

/// A type that can do something with information about differences in a
/// sequence, like a slice or `Vec`.
pub trait SeqDiffer {
    /// Type returned on success.
    type Ok;
    /// Type returned on failure.
    type Err;

    /// We've found elements in corresponding positions in both sequences.
    fn diff_element<T: ?Sized>(&mut self, a: &T, b: &T)
    where
        T: Diff;

    /// We've found an element that only appears in the left-hand sequence.
    fn left_excess<T: ?Sized>(&mut self, a: &T)
    where
        T: Diff;

    /// We've found an element that only appears in the right-hand sequence.
    fn right_excess<T: ?Sized>(&mut self, b: &T)
    where
        T: Diff;

    /// Consumes two iterators, diffing their contents. This is a convenience
    /// method implemented in terms of the others.
    fn diff_elements<T, I>(&mut self, a: I, b: I)
    where
        T: Diff,
        I: IntoIterator<Item = T>,
    {
        for ab in a.into_iter().zip_longest(b) {
            match ab {
                EitherOrBoth::Both(a, b) => self.diff_element(&a, &b),
                EitherOrBoth::Left(a) => self.left_excess(&a),
                EitherOrBoth::Right(b) => self.right_excess(&b),
            }
        }
    }

    /// Complete the sequence and produce the result.
    fn end(self) -> Result<Self::Ok, Self::Err>;
}

/// A type that can do something with information about differences in a
/// map-like, key-value type.
pub trait MapDiffer {
    /// Type returned on success.
    type Ok;
    /// Type returned on failure.
    type Err;

    /// Both maps contain entries for `key`; check them for differences.
    fn diff_entry<K, V>(&mut self, key: &K, a: &V, b: &V)
    where
        K: ?Sized + Debug,
        V: ?Sized + Diff;

    /// Key `key` is only present in the left map, with value `a`.
    fn only_in_left<K, V>(&mut self, key: &K, a: &V)
    where
        K: ?Sized + Debug,
        V: ?Sized + Diff;

    /// Key `key` is only present in the right map, with value `b`.
    fn only_in_right<K, V>(&mut self, key: &K, b: &V)
    where
        K: ?Sized + Debug,
        V: ?Sized + Diff;

    /// We've reached the end of the maps.
    fn end(self) -> Result<Self::Ok, Self::Err>;
}

/// A type that can do something with information about differences in a
/// set-like sequence type, i.e. one in which elements are expected to be
/// unique.
pub trait SetDiffer {
    /// Type returned on success.
    type Ok;
    /// Type returned on failure.
    type Err;

    /// The sets contain `a` and `b` which compare as equal. Check them for
    /// differences.
    fn diff_equal<V>(&mut self, a: &V, b: &V)
    where
        V: ?Sized + Diff;

    /// Value `a` is only in the left-hand set.
    fn only_in_left<V>(&mut self, a: &V)
    where
        V: ?Sized + Diff;

    /// Value `b` is only in the right-hand set.
    fn only_in_right<V>(&mut self, b: &V)
    where
        V: ?Sized + Diff;

    /// We've reached the end of the sets.
    fn end(self) -> Result<Self::Ok, Self::Err>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug)]
    pub enum TestEnum {
        First,
        Second,
        Struct { a: usize, b: bool },
    }

    impl Diff for TestEnum {
        fn diff<D>(a: &Self, b: &Self, out: D) -> Result<D::Ok, D::Err>
        where
            D: Differ,
        {
            match (a, b) {
                (TestEnum::First, TestEnum::First) => out.same(a, b),
                (TestEnum::Second, TestEnum::Second) => out.same(a, b),
                (
                    TestEnum::Struct { a: aa, b: ab },
                    TestEnum::Struct { a: ba, b: bb },
                ) => {
                    let mut s = out.begin_struct_variant("TestEnum", "Struct");
                    s.diff_field("a", &aa, &ba);
                    s.diff_field("b", &ab, &bb);
                    s.end()
                }
                _ => out.difference(a, b),
            }
        }
    }

    #[derive(Clone, Debug)]
    pub struct TestStruct {
        pub distance: usize,
        pub silly: bool,
    }

    impl Diff for TestStruct {
        fn diff<D>(a: &Self, b: &Self, out: D) -> Result<D::Ok, D::Err>
        where
            D: Differ,
        {
            let mut s = out.begin_struct("TestStruct");
            s.diff_field("distance", &a.distance, &b.distance);
            s.diff_field("silly", &a.silly, &b.silly);
            s.end()
        }
    }
}
