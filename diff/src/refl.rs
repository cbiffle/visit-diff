pub trait Reflect {
    fn reflect<M>(&self, mirror: M) -> Result<M::Ok, M::Error>
    where
        M: Mirror;
}

impl<T> Reflect for &T
where
    T: ?Sized + Reflect,
{
    fn reflect<M>(&self, mirror: M) -> Result<M::Ok, M::Error>
    where
        M: Mirror,
    {
        (*self).reflect(mirror)
    }
}

pub trait Mirror {
    type Ok;
    type Error;

    type StructMirror: StructMirror<Ok = Self::Ok, Error = Self::Error>;

    fn reflect_bool(self, v: bool) -> Result<Self::Ok, Self::Error>;

    fn reflect_unit(self) -> Result<Self::Ok, Self::Error>;

    fn reflect_newtype<T>(
        self,
        ty: &'static str,
        content: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Reflect;

    fn reflect_struct(
        self,
        ty: &'static str,
        field_count: usize,
    ) -> Result<Self::StructMirror, Self::Error>;
}

pub trait StructMirror {
    type Ok;
    type Error;

    fn field<T>(
        &mut self,
        name: &'static str,
        val: &T,
    ) -> Result<(), Self::Error>
    where
        T: ?Sized + Reflect;

    fn end(self) -> Result<Self::Ok, Self::Error>;
}

////////////

impl Reflect for bool {
    fn reflect<M>(&self, mirror: M) -> Result<M::Ok, M::Error>
    where
        M: Mirror,
    {
        mirror.reflect_bool(*self)
    }
}

impl Reflect for () {
    fn reflect<M>(&self, mirror: M) -> Result<M::Ok, M::Error>
    where
        M: Mirror,
    {
        mirror.reflect_unit()
    }
}

////////////

struct DebugMirror<'a, 'b>(&'a mut core::fmt::Formatter<'b>);

impl<'a, 'b> Mirror for DebugMirror<'a, 'b> {
    type Ok = ();
    type Error = core::fmt::Error;

    type StructMirror = DebugStructMirror<'a, 'b>;

    fn reflect_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        <bool as core::fmt::Debug>::fmt(&v, self.0)
    }

    fn reflect_unit(self) -> Result<Self::Ok, Self::Error> {
        <() as core::fmt::Debug>::fmt(&(), self.0)
    }

    fn reflect_newtype<T>(
        self,
        ty: &'static str,
        content: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Reflect,
    {
        self.0
            .debug_tuple(ty)
            .field(&DebugAdapter(content))
            .finish()
    }

    fn reflect_struct(
        self,
        ty: &'static str,
        _: usize,
    ) -> Result<Self::StructMirror, Self::Error> {
        Ok(DebugStructMirror(self.0.debug_struct(ty)))
    }
}

struct DebugStructMirror<'a, 'b>(core::fmt::DebugStruct<'a, 'b>);

impl<'a, 'b> StructMirror for DebugStructMirror<'a, 'b> {
    type Ok = ();
    type Error = core::fmt::Error;

    fn field<T>(
        &mut self,
        name: &'static str,
        val: &T,
    ) -> Result<(), Self::Error>
    where
        T: ?Sized + Reflect,
    {
        self.0.field(name, &DebugAdapter(val));
        Ok(())
    }

    fn end(mut self) -> Result<Self::Ok, Self::Error> {
        self.0.finish()
    }
}

/// Adapts any `Reflect` as `Debug`.
struct DebugAdapter<T: ?Sized>(pub T);

impl<T> core::fmt::Debug for DebugAdapter<T>
where
    T: ?Sized + Reflect,
{
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        self.0.reflect(DebugMirror(fmt))
    }
}

/// Adapts any `Reflect` as `Debug`.
pub fn make_debug<T>(value: T) -> impl core::fmt::Debug
where
    T: Reflect,
{
    DebugAdapter(value)
}

///////////

pub fn make_serialize<T: Reflect>(value: T) -> impl serde::Serialize {
    SerializeAdapter(value)
}

/// Adapts any `Reflect` as `Serialize`.
struct SerializeAdapter<T: ?Sized>(pub T);

impl<T> serde::Serialize for SerializeAdapter<T>
where
    T: ?Sized + Reflect,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.reflect(SerializeMirror(serializer))
    }
}

struct SerializeMirror<S>(S);

impl<S> Mirror for SerializeMirror<S>
where
    S: serde::Serializer,
{
    type Ok = S::Ok;
    type Error = S::Error;

    type StructMirror = SerializeStructMirror<S::SerializeStruct>;

    fn reflect_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.0.serialize_bool(v)
    }

    fn reflect_unit(self) -> Result<Self::Ok, Self::Error> {
        self.0.serialize_unit()
    }

    fn reflect_newtype<T>(
        self,
        ty: &'static str,
        content: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Reflect,
    {
        self.0
            .serialize_newtype_struct(ty, &SerializeAdapter(content))
    }

    fn reflect_struct(
        self,
        ty: &'static str,
        field_count: usize,
    ) -> Result<Self::StructMirror, Self::Error> {
        self.0
            .serialize_struct(ty, field_count)
            .map(SerializeStructMirror)
    }
}

struct SerializeStructMirror<S>(S);

impl<S> StructMirror for SerializeStructMirror<S>
where
    S: serde::ser::SerializeStruct,
{
    type Ok = S::Ok;
    type Error = S::Error;

    fn field<T>(
        &mut self,
        name: &'static str,
        val: &T,
    ) -> Result<(), Self::Error>
    where
        T: ?Sized + Reflect,
    {
        self.0.serialize_field(name, &SerializeAdapter(val))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.0.end()
    }
}

//////////

pub enum Gen {
    Unit,
    Bool(bool),
    Newtype(&'static str, Box<Gen>),
    Struct(&'static str, Struct),
}

pub struct Struct {
    pub fields: Vec<(&'static str, Gen)>,
}

struct GenMirror;

impl Mirror for GenMirror {
    type Ok = Gen;
    type Error = ();

    type StructMirror = GenStructMirror;

    fn reflect_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Ok(Gen::Bool(v))
    }

    fn reflect_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(Gen::Unit)
    }

    fn reflect_newtype<T>(
        self,
        ty: &'static str,
        content: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Reflect,
    {
        Ok(Gen::Newtype(ty, Box::new(content.reflect(GenMirror)?)))
    }

    fn reflect_struct(
        self,
        ty: &'static str,
        field_count: usize,
    ) -> Result<Self::StructMirror, Self::Error> {
        Ok(GenStructMirror {
            name: ty,
            fields: Vec::with_capacity(field_count),
        })
    }
}

struct GenStructMirror {
    name: &'static str,
    fields: Vec<(&'static str, Gen)>,
}

impl StructMirror for GenStructMirror {
    type Ok = Gen;
    type Error = ();

    fn field<T>(
        &mut self,
        name: &'static str,
        val: &T,
    ) -> Result<(), Self::Error>
    where
        T: ?Sized + Reflect,
    {
        self.fields.push((name, val.reflect(GenMirror)?));
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Gen::Struct(
            self.name,
            Struct {
                fields: self.fields,
            },
        ))
    }
}

//////////

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Copy, Clone, Debug)]
    struct TestStruct {
        a: bool,
        b: (),
    }

    impl Reflect for TestStruct {
        fn reflect<M>(&self, mirror: M) -> Result<M::Ok, M::Error>
        where
            M: Mirror,
        {
            let mut s = mirror.reflect_struct("TestStruct", 2)?;
            s.field("a", &self.a)?;
            s.field("b", &self.b)?;
            s.end()
        }
    }

    /// Confirms that the `Debug` instance generated for `DebugAdapter<T>`
    /// produces the same results as the native derived instance.
    ///
    /// This is interesting because `DebugAdapter` bypasses the `Debug` instance
    /// for `T`.
    #[test]
    fn debug() {
        let a = TestStruct { a: true, b: () };

        assert_eq!(format!("{:?}", DebugAdapter(a)), format!("{:?}", a),);

        assert_eq!(format!("{:#?}", DebugAdapter(a)), format!("{:#?}", a),);
    }
}
