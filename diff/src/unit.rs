use core::fmt::Debug;
use void::Void;

use crate::{
    Diff, Differ, MapDiffer, SeqDiffer, SetDiffer, StructDiffer, TupleDiffer,
};

impl Differ for () {
    type Ok = ();
    type Err = Void;

    type StructDiffer = ();
    type StructVariantDiffer = ();
    type TupleDiffer = ();
    type TupleVariantDiffer = ();
    type SeqDiffer = ();
    type MapDiffer = ();
    type SetDiffer = ();

    fn difference(self, _: &Debug, _: &Debug) -> Result<Self::Ok, Self::Err> {
        Ok(())
    }

    fn same(self, _: &Debug, _: &Debug) -> Result<Self::Ok, Self::Err> {
        Ok(())
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
        Ok(())
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

impl StructDiffer for () {
    type Ok = ();
    type Err = Void;

    fn diff_field<T: ?Sized>(&mut self, _: &'static str, _: &T, _: &T)
    where
        T: Diff,
    {
    }

    fn end(self) -> Result<Self::Ok, Self::Err> {
        Ok(self)
    }
}

impl TupleDiffer for () {
    type Ok = ();
    type Err = Void;

    fn diff_field<T: ?Sized>(&mut self, _: &T, _: &T)
    where
        T: Diff,
    {
    }

    fn end(self) -> Result<Self::Ok, Self::Err> {
        Ok(self)
    }
}

impl SeqDiffer for () {
    type Ok = ();
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
        Ok(self)
    }
}

impl SetDiffer for () {
    type Ok = ();
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
        Ok(self)
    }
}

impl MapDiffer for () {
    type Ok = ();
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
        Ok(self)
    }
}
