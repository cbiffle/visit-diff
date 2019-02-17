//! A do-nothing differ that always winds up returning a fixed value.
//!
//! This module is mostly useful during development of a new `Differ`: there are
//! a lot of associated types and functions required to implement `Differ`, but
//! it's nice to see your code compiling part way through. You can stub out
//! parts of `Differ` by deferring to `Const`. For example, if your `Differ`
//! returns a `u32` and you haven't implemented map diffing yet, you can use the
//! associated type
//!
//! ```ignore
//! type MapDiffer = visit_diff::constant::Const<u32>;
//! ```
//!
//! and then stub out `begin_map` like so:
//!
//! ```ignore
//! fn begin_map(self) -> Self::MapDiffer {
//!     visit_diff::constant::Const(4)
//! }
//! ```
use core::fmt::Debug;
use void::Void;

use crate::{
    Diff, Differ, MapDiffer, SeqDiffer, SetDiffer, StructDiffer, TupleDiffer,
};

pub struct Const<R>(pub R);

impl<R> Differ for Const<R> {
    type Ok = R;
    type Err = Void;

    type StructDiffer = Self;
    type StructVariantDiffer = Self;
    type TupleDiffer = Self;
    type TupleVariantDiffer = Self;
    type SeqDiffer = Self;
    type MapDiffer = Self;
    type SetDiffer = Self;

    fn difference(self, _: &Debug, _: &Debug) -> Result<Self::Ok, Self::Err> {
        Ok(self.0)
    }

    fn same(self, _: &Debug, _: &Debug) -> Result<Self::Ok, Self::Err> {
        Ok(self.0)
    }

    fn diff_newtype<T: ?Sized>(
        self,
        _: &'static str,
        _: &T,
        _: &T,
    ) -> Result<Self::Ok, Self::Err>
    where
        T: Diff,
    {
        Ok(self.0)
    }

    /// Begin traversing a struct.
    fn begin_struct(self, _: &'static str) -> Self::StructDiffer {
        self
    }

    fn begin_struct_variant(
        self,
        _: &'static str,
        _: &'static str,
    ) -> Self::StructVariantDiffer {
        self
    }

    fn begin_tuple(self, _: &'static str) -> Self::TupleDiffer {
        self
    }

    fn begin_tuple_variant(
        self,
        _: &'static str,
        _: &'static str,
    ) -> Self::TupleVariantDiffer {
        self
    }

    fn begin_seq(self) -> Self::SeqDiffer {
        self
    }

    fn begin_map(self) -> Self::MapDiffer {
        self
    }

    fn begin_set(self) -> Self::SetDiffer {
        self
    }
}

impl<R> StructDiffer for Const<R> {
    type Ok = R;
    type Err = Void;

    fn diff_field<T: ?Sized>(&mut self, _: &'static str, _: &T, _: &T)
    where
        T: Diff,
    {
    }

    fn end(self) -> Result<Self::Ok, Self::Err> {
        Ok(self.0)
    }
}

impl<R> TupleDiffer for Const<R> {
    type Ok = R;
    type Err = Void;

    fn diff_field<T: ?Sized>(&mut self, _: &T, _: &T)
    where
        T: Diff,
    {
    }

    fn end(self) -> Result<Self::Ok, Self::Err> {
        Ok(self.0)
    }
}

impl<R> SeqDiffer for Const<R> {
    type Ok = R;
    type Err = Void;

    fn diff_element<T: ?Sized>(&mut self, _: &T, _: &T)
    where
        T: Diff,
    {
    }

    fn diff_elements<T, I>(&mut self, _: I, _: I)
    where
        T: Diff,
        I: IntoIterator<Item = T>,
    {
    }

    fn left_excess<T: ?Sized>(&mut self, _: &T)
    where
        T: Diff,
    {
    }

    fn right_excess<T: ?Sized>(&mut self, _: &T)
    where
        T: Diff,
    {
    }

    fn end(self) -> Result<Self::Ok, Self::Err> {
        Ok(self.0)
    }
}

impl<R> SetDiffer for Const<R> {
    type Ok = R;
    type Err = Void;

    fn diff_equal<V>(&mut self, _: &V, _: &V)
    where
        V: ?Sized + Diff,
    {
    }

    fn only_in_left<V>(&mut self, _: &V)
    where
        V: ?Sized + Diff,
    {
    }

    fn only_in_right<V>(&mut self, _: &V)
    where
        V: ?Sized + Diff,
    {
    }

    fn end(self) -> Result<Self::Ok, Self::Err> {
        Ok(self.0)
    }
}

impl<R> MapDiffer for Const<R> {
    type Ok = R;
    type Err = Void;

    fn diff_entry<K, V>(&mut self, _: &K, _: &V, _: &V)
    where
        K: ?Sized + Debug,
        V: ?Sized + Diff,
    {
    }

    fn only_in_left<K, V>(&mut self, _: &K, _: &V)
    where
        K: ?Sized + Debug,
        V: ?Sized + Diff,
    {
    }

    fn only_in_right<K, V>(&mut self, _: &K, _: &V)
    where
        K: ?Sized + Debug,
        V: ?Sized + Diff,
    {
    }

    fn end(self) -> Result<Self::Ok, Self::Err> {
        Ok(self.0)
    }
}
