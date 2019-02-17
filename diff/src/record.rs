//! Records a diff in a general data structure that can then be analyzed without
//! knowledge of the original types involved.
//!
//! This is particularly useful when testing a `Diff` implementation separately
//! from any particular `Differ`, but you might find other uses for it.

use std::fmt::Debug;
use void::{ResultVoidExt, Void};

use crate::{Diff, Differ, StructDiffer, TupleDiffer, SeqDiffer, SetDiffer, MapDiffer};

/// Produces a `Value` describing differences between `a` and `b`.
pub fn record_diff<T: Diff>(a: &T, b: &T) -> Value {
    Diff::diff(a, b, ValueRecorder).void_unwrap()
}

/// A representation of differences between two values of a single Rust type.
///
/// Atomic values are flattened into `String` using their `Debug`
/// implementation, but everything else is represented as a structure you can
/// examine.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Value {
    /// Two atomic values that were reported as equivalent, in Debug format.
    Same(String, String),
    /// Two atomic values that were reported as different, in Debug format.
    Difference(String, String),
    /// A newtype.
    Newtype(&'static str, Box<Value>),
    /// A struct type.
    Struct(Struct),
    /// A tuple or tuple struct type.
    Tuple(Tuple),
    /// An enumerated type.
    Enum(Enum),
    /// An abstract sequence, such as a vector or slice.
    Sequence(Vec<Element>),
    /// An abstract set.
    Set(Vec<Element>),
    /// An abstract map.
    Map(Vec<(String, Element)>),
}

/// Representation of differences between two structs of a common type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Struct {
    /// Name of the struct: type name for standalone struct, or variant name for
    /// enum struct-variants.
    pub name: &'static str,
    /// Fields of the struct in the order they were visited. Fields visited
    /// using [`skip_field`] have the value `None`, everything else is `Some`.
    ///
    /// [`skip_field`]: ../trait.StructDiffer.html#method.skip_field
    pub fields: Vec<(&'static str, Option<Value>)>,
}

/// Representation of differences between two tuples of a common type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Tuple {
    /// Name of the tuple: type name for a tuple struct, variant name for enum
    /// tuple-variants, or the empty string for a raw tuple.
    pub name: &'static str,
    /// Fields of the tuple in order. Fields visited using [`skip_field`] have
    /// the value `None`, everything else is `Some`.
    ///
    /// [`skip_field`]: ../trait.TupleDiffer.html#method.skip_field
    pub fields: Vec<Option<Value>>,
}

/// Representation of differences between two values of an enum type that use
/// the *same* discriminator.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Enum {
    /// Name of the enum type.
    pub name: &'static str,
    /// Shape of the variant.
    pub variant: Variant,
}

/// Shape of an enum variant.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Variant {
    /// A struct-variant.
    Struct(Struct),
    /// A tuple-variant.
    Tuple(Tuple),
}

/// Difference between two sequences or sets at a single position.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Element {
    /// A flattened value appears only in the left-hand sequence.
    LeftOnly(String),
    /// A flattened value appears only in the right-hand sequence.
    RightOnly(String),
    /// Both sequences contain a value at this position, so the differences will
    /// be more finely specified.
    Both(Value),
}

struct ValueRecorder;

impl Differ for ValueRecorder {
    type Ok = Value;
    type Err = Void;

    type StructDiffer = StructRecorder;
    type StructVariantDiffer = StructRecorder;
    type TupleDiffer = TupleRecorder;
    type TupleVariantDiffer = TupleRecorder;
    type SeqDiffer = SequenceRecorder;
    type MapDiffer = MapRecorder;
    type SetDiffer = SequenceRecorder;

    fn difference(self, a: &Debug, b: &Debug) -> Result<Self::Ok, Self::Err> {
        Ok(Value::Difference(format!("{:?}", a), format!("{:?}", b)))
    }

    fn same(self, a: &Debug, b: &Debug) -> Result<Self::Ok, Self::Err> {
        Ok(Value::Same(format!("{:?}", a), format!("{:?}", b)))
    }

    /// Encounter a newtype. `a` and `b` are the contents of the sole fields of
    /// the left-hand and right-hand value, respectively.
    fn diff_newtype<T: ?Sized>(
        self,
        ty: &'static str,
        a: &T,
        b: &T,
    ) -> Result<Self::Ok, Self::Err>
    where
        T: Diff,
    {
        Ok(Value::Newtype(
            ty,
            Box::new(Diff::diff(a, b, ValueRecorder).void_unwrap()),
        ))
    }

    fn begin_struct(self, ty: &'static str) -> Self::StructDiffer {
        StructRecorder(Struct {
            name: ty,
            fields: vec![],
        }, OutputStyle::Raw)
    }

    fn begin_struct_variant(
        self,
        ty: &'static str,
        var: &'static str,
    ) -> Self::StructVariantDiffer {
        StructRecorder(Struct {
            name: var,
            fields: vec![],
        }, OutputStyle::VariantOf(ty))
    }

    fn begin_tuple(self, ty: &'static str) -> Self::TupleDiffer {
        TupleRecorder(Tuple {
            name: ty,
            fields: vec![],
        }, OutputStyle::Raw)
    }

    fn begin_tuple_variant(
        self,
        ty: &'static str,
        var: &'static str,
    ) -> Self::TupleVariantDiffer {
        TupleRecorder(Tuple {
            name: var,
            fields: vec![],
        }, OutputStyle::VariantOf(ty))
    }

    fn begin_seq(self) -> Self::SeqDiffer {
        SequenceRecorder(vec![])
    }

    fn begin_map(self) -> Self::MapDiffer {
        MapRecorder(vec![])
    }

    /// Begin traversing a set.
    fn begin_set(self) -> Self::SetDiffer {
        SequenceRecorder(vec![])
    }
}

enum OutputStyle {
    Raw,
    VariantOf(&'static str),
}

struct StructRecorder(Struct, OutputStyle);

impl StructDiffer for StructRecorder {
    type Ok = Value;
    type Err = Void;

    fn diff_field<T: ?Sized>(&mut self, name: &'static str, a: &T, b: &T)
    where
        T: Diff,
    {
        let val = Diff::diff(a, b, ValueRecorder).void_unwrap();
        self.0.fields.push((name, Some(val)))
    }

    fn skip_field<T: ?Sized>(&mut self, name: &'static str) {
        self.0.fields.push((name, None))
    }

    fn end(self) -> Result<Self::Ok, Self::Err> {
        match self.1 {
            OutputStyle::Raw => Ok(Value::Struct(self.0)),
            OutputStyle::VariantOf(ty) => Ok(Value::Enum(Enum {
                name: ty,
                variant: Variant::Struct(self.0),
            })),
        }
    }
}

struct TupleRecorder(Tuple, OutputStyle);

impl TupleDiffer for TupleRecorder {
    type Ok = Value;
    type Err = Void;

    fn diff_field<T: ?Sized>(&mut self, a: &T, b: &T)
    where
        T: Diff,
    {
        let val = Diff::diff(a, b, ValueRecorder).void_unwrap();
        self.0.fields.push(Some(val))
    }

    fn skip_field<T: ?Sized>(&mut self) {
        self.0.fields.push(None)
    }

    fn end(self) -> Result<Self::Ok, Self::Err> {
        match self.1 {
            OutputStyle::Raw => Ok(Value::Tuple(self.0)),
            OutputStyle::VariantOf(ty) => Ok(Value::Enum(Enum {
                name: ty,
                variant: Variant::Tuple(self.0),
            })),
        }
    }
}

struct SequenceRecorder(Vec<Element>);

impl SeqDiffer for SequenceRecorder {
    type Ok = Value;
    type Err = Void;

    fn diff_element<T: ?Sized>(&mut self, a: &T, b: &T)
    where
        T: Diff
    {
        self.0.push(Element::Both(Diff::diff(a, b, ValueRecorder).void_unwrap()))
    }

    fn left_excess<T: ?Sized>(&mut self, a: &T)
    where
        T: Diff
    {
        self.0.push(Element::LeftOnly(format!("{:?}", a)))
    }

    fn right_excess<T: ?Sized>(&mut self, a: &T)
    where
        T: Diff
    {
        self.0.push(Element::RightOnly(format!("{:?}", a)))
    }

    fn end(self) -> Result<Self::Ok, Self::Err> {
        Ok(Value::Sequence(self.0))
    }
}

impl SetDiffer for SequenceRecorder {
    type Ok = Value;
    type Err = Void;

    fn diff_equal<T: ?Sized>(&mut self, a: &T, b: &T)
    where
        T: Diff
    {
        self.0.push(Element::Both(Diff::diff(a, b, ValueRecorder).void_unwrap()))
    }

    fn only_in_left<T: ?Sized>(&mut self, a: &T)
    where
        T: Diff
    {
        self.0.push(Element::LeftOnly(format!("{:?}", a)))
    }

    fn only_in_right<T: ?Sized>(&mut self, a: &T)
    where
        T: Diff
    {
        self.0.push(Element::RightOnly(format!("{:?}", a)))
    }

    fn end(self) -> Result<Self::Ok, Self::Err> {
        Ok(Value::Set(self.0))
    }
}

struct MapRecorder(Vec<(String, Element)>);

impl MapDiffer for MapRecorder {
    type Ok = Value;
    type Err = Void;

    fn diff_entry<K, V>(&mut self, key: &K, a: &V, b: &V)
    where
        K: ?Sized + Debug,
        V: ?Sized + Diff,
    {
        let key = format!("{:?}", key);
        let diff = Diff::diff(a, b, ValueRecorder).void_unwrap();
        self.0.push((key, Element::Both(diff)))
    }

    fn only_in_left<K, V>(&mut self, key: &K, a: &V)
    where
        K: ?Sized + Debug,
        V: ?Sized + Diff,
    {
        let key = format!("{:?}", key);
        self.0.push((key, Element::LeftOnly(format!("{:?}", a))))
    }

    fn only_in_right<K, V>(&mut self, key: &K, a: &V)
    where
        K: ?Sized + Debug,
        V: ?Sized + Diff,
    {
        let key = format!("{:?}", key);
        self.0.push((key, Element::RightOnly(format!("{:?}", a))))
    }

    fn end(self) -> Result<Self::Ok, Self::Err> {
        Ok(Value::Map(self.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit() {
        let diff = Diff::diff(&(), &(), ValueRecorder).void_unwrap();
        assert_eq!(diff, Value::Same("()".into(), "()".into()));
    }

    #[test]
    fn int() {
        let diff = Diff::diff(&0u32, &0, ValueRecorder).void_unwrap();
        assert_eq!(diff, Value::Same("0".into(), "0".into()));

        let diff = Diff::diff(&0u32, &1, ValueRecorder).void_unwrap();
        assert_eq!(diff, Value::Difference("0".into(), "1".into()));
    }
}
