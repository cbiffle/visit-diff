use super::*;

////////////////////////////////////////////////////////////////////////////////
// Unit-shaped things

impl Diff for () {
    fn diff<D>(a: &Self, b: &Self, out: D) -> Result<D::Ok, D::Err>
    where
        D: Differ,
    {
        out.same(a, b)
    }
}

impl<T: ?Sized> Diff for core::marker::PhantomData<T> {
    fn diff<D>(a: &Self, b: &Self, out: D) -> Result<D::Ok, D::Err>
    where
        D: Differ,
    {
        out.same(a, b)
    }
}

////////////////////////////////////////////////////////////////////////////////
// Tuple boilerplate

macro_rules! tuple_impl {
    ($($p:ident / $n:tt),*) => {
        impl<$($p),*> Diff for ($($p,)*)
        where
            $($p: Diff),*
        {
            fn diff<DD>(a: &Self, b: &Self, out: DD) -> Result<DD::Ok, DD::Err>
            where
                DD: Differ,
            {
                let mut out = out.begin_tuple("");
                $(out.diff_field(&a.$n, &b.$n);)*
                out.end()
            }
        }
    };
}

tuple_impl!(A / 0);
tuple_impl!(A / 0, B / 1);
tuple_impl!(A / 0, B / 1, C / 2);
tuple_impl!(A / 0, B / 1, C / 2, D / 3);
tuple_impl!(A / 0, B / 1, C / 2, D / 3, E / 4);
tuple_impl!(A / 0, B / 1, C / 2, D / 3, E / 4, F / 5);
tuple_impl!(A / 0, B / 1, C / 2, D / 3, E / 4, F / 5, G / 6);
tuple_impl!(A / 0, B / 1, C / 2, D / 3, E / 4, F / 5, G / 6, H / 7);
tuple_impl!(
    A / 0,
    B / 1,
    C / 2,
    D / 3,
    E / 4,
    F / 5,
    G / 6,
    H / 7,
    I / 8
);

////////////////////////////////////////////////////////////////////////////////
// Slice and array boilerplate

impl<T> Diff for [T]
where
    T: Diff,
{
    fn diff<D>(a: &Self, b: &Self, out: D) -> Result<D::Ok, D::Err>
    where
        D: Differ,
    {
        let mut s = out.begin_seq();
        s.diff_elements(a, b);
        s.end()
    }
}

macro_rules! array_impl {
    ($n:tt) => {
        impl<T> Diff for [T; $n]
        where
            T: Diff,
        {
            fn diff<D>(a: &Self, b: &Self, out: D) -> Result<D::Ok, D::Err>
            where
                D: Differ,
            {
                Diff::diff(a as &[T], b as &[T], out)
            }
        }
    };
}

array_impl!(0);
array_impl!(1);
array_impl!(2);
array_impl!(3);
array_impl!(4);
array_impl!(5);
array_impl!(6);
array_impl!(7);
array_impl!(8);
array_impl!(9);
array_impl!(10);
array_impl!(11);
array_impl!(12);
array_impl!(13);
array_impl!(14);
array_impl!(15);
array_impl!(16);
array_impl!(17);
array_impl!(18);
array_impl!(19);
array_impl!(20);
array_impl!(21);
array_impl!(22);
array_impl!(23);
array_impl!(24);
array_impl!(25);
array_impl!(26);
array_impl!(27);
array_impl!(28);
array_impl!(29);
array_impl!(30);
array_impl!(31);
array_impl!(32);

////////////////////////////////////////////////////////////////////////////////
// References

/// Diff references by dereferencing.
impl<T> Diff for &T
where
    T: Diff + ?Sized,
{
    fn diff<D>(a: &Self, b: &Self, out: D) -> Result<D::Ok, D::Err>
    where
        D: Differ,
    {
        Diff::diff(*a, *b, out)
    }
}

/// Diff references by dereferencing.
impl<T> Diff for &mut T
where
    T: Diff + ?Sized,
{
    fn diff<D>(a: &Self, b: &Self, out: D) -> Result<D::Ok, D::Err>
    where
        D: Differ,
    {
        Diff::diff(*a, *b, out)
    }
}

impl<'a, T: ?Sized + Diff> Diff for core::cell::Ref<'a, T> {
    fn diff<D>(a: &Self, b: &Self, out: D) -> Result<D::Ok, D::Err>
    where
        D: Differ,
    {
        Diff::diff(&**a, &**b, out)
    }
}

impl<'a, T: ?Sized + Diff> Diff for core::cell::RefMut<'a, T> {
    fn diff<D>(a: &Self, b: &Self, out: D) -> Result<D::Ok, D::Err>
    where
        D: Differ,
    {
        Diff::diff(&**a, &**b, out)
    }
}

////////////////////////////////////////////////////////////////////////////////
// "Atomic" types that can be diffed using PartialEq.

macro_rules! impl_diff_partial_eq {
    // Specialization for unsized types: adds a reference on the way to the
    // differ.
    (unsized $ty:ty) => {
        impl Diff for $ty {
            fn diff<D>(a: &Self, b: &Self, out: D) -> Result<D::Ok, D::Err>
            where
                D: Differ,
            {
                if a != b {
                    out.difference(&a, &b)
                } else {
                    out.same(&a, &b)
                }
            }
        }
    };
    ($ty:ty | $p:ident) => {
        impl<$p> Diff for $ty
        where
            $ty: PartialEq + Debug,
        {
            fn diff<D>(a: &Self, b: &Self, out: D) -> Result<D::Ok, D::Err>
            where
                D: Differ,
            {
                if a != b {
                    out.difference(&a, &b)
                } else {
                    out.same(&a, &b)
                }
            }
        }
    };
    ($ty:ty) => {
        impl Diff for $ty {
            fn diff<D>(a: &Self, b: &Self, out: D) -> Result<D::Ok, D::Err>
            where
                D: Differ,
            {
                if a != b {
                    out.difference(a, b)
                } else {
                    out.same(a, b)
                }
            }
        }
    };
}

impl_diff_partial_eq!(bool);
impl_diff_partial_eq!(char);
impl_diff_partial_eq!(u8);
impl_diff_partial_eq!(u16);
impl_diff_partial_eq!(u32);
impl_diff_partial_eq!(u64);
impl_diff_partial_eq!(u128);
impl_diff_partial_eq!(usize);
impl_diff_partial_eq!(i8);
impl_diff_partial_eq!(i16);
impl_diff_partial_eq!(i32);
impl_diff_partial_eq!(i64);
impl_diff_partial_eq!(i128);
impl_diff_partial_eq!(isize);
impl_diff_partial_eq!(f32);
impl_diff_partial_eq!(f64);
impl_diff_partial_eq!(unsized str);
impl_diff_partial_eq!(core::cmp::Ordering);
impl_diff_partial_eq!(core::time::Duration);

// Ranges are treated as atomic values in this version, because they have
// strange Debug impls that would otherwise require explicit support in the
// Differ traits.
impl_diff_partial_eq!(core::ops::Range<T> | T);
impl_diff_partial_eq!(core::ops::RangeFrom<T> | T);
impl_diff_partial_eq!(core::ops::RangeFull);
impl_diff_partial_eq!(core::ops::RangeTo<T> | T);
impl_diff_partial_eq!(core::ops::RangeToInclusive<T> | T);

/// Pointers diff by address, not by contents.
impl<T: ?Sized> Diff for *const T {
    fn diff<D>(a: &Self, b: &Self, out: D) -> Result<D::Ok, D::Err>
    where
        D: Differ,
    {
        if a != b {
            out.difference(a, b)
        } else {
            out.same(a, b)
        }
    }
}

/// Pointers diff by address, not by contents.
impl<T: ?Sized> Diff for *mut T {
    fn diff<D>(a: &Self, b: &Self, out: D) -> Result<D::Ok, D::Err>
    where
        D: Differ,
    {
        if a != b {
            out.difference(a, b)
        } else {
            out.same(a, b)
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Trivial containers and cells. The trivial containers in core vary on whether
// they want to be represented as a simple newtype, or as a struct containing a
// single field. We follow their Debug implementations.

impl<T: Copy + Diff> Diff for core::cell::Cell<T> {
    fn diff<D>(a: &Self, b: &Self, out: D) -> Result<D::Ok, D::Err>
    where
        D: Differ,
    {
        let mut out = out.begin_struct("Cell");
        out.diff_field("value", &a.get(), &b.get());
        out.end()
    }
}

impl<T: ?Sized + Diff> Diff for core::mem::ManuallyDrop<T> {
    fn diff<D>(a: &Self, b: &Self, out: D) -> Result<D::Ok, D::Err>
    where
        D: Differ,
    {
        let mut out = out.begin_struct("ManuallyDrop");
        out.diff_field("value", &*a, &*b);
        out.end()
    }
}

impl<T: Diff> Diff for core::num::Wrapping<T> {
    fn diff<D>(a: &Self, b: &Self, out: D) -> Result<D::Ok, D::Err>
    where
        D: Differ,
    {
        Diff::diff(&a.0, &b.0, out)
    }
}

/// Note that this *will* panic if the RefCell is mutably borrowed.
impl<T: ?Sized + Diff> Diff for core::cell::RefCell<T> {
    fn diff<D>(a: &Self, b: &Self, out: D) -> Result<D::Ok, D::Err>
    where
        D: Differ,
    {
        let mut out = out.begin_struct("RefCell");
        out.diff_field("value", &*a.borrow(), &*b.borrow());
        out.end()
    }
}

impl<T: Diff> Diff for core::option::Option<T> {
    fn diff<D>(a: &Self, b: &Self, out: D) -> Result<D::Ok, D::Err>
    where
        D: Differ,
    {
        match (a, b) {
            (None, None) => out.same(a, b),
            (Some(a), Some(b)) => {
                let mut out = out.begin_tuple_variant("Option", "Some");
                out.diff_field(a, b);
                out.end()
            }
            _ => out.difference(a, b),
        }
    }
}

impl<T: Diff, E: Diff> Diff for core::result::Result<T, E> {
    fn diff<D>(a: &Self, b: &Self, out: D) -> Result<D::Ok, D::Err>
    where
        D: Differ,
    {
        match (a, b) {
            (Ok(a), Ok(b)) => {
                let mut out = out.begin_tuple_variant("Result", "Ok");
                out.diff_field(a, b);
                out.end()
            }
            (Err(a), Err(b)) => {
                let mut out = out.begin_tuple_variant("Result", "Err");
                out.diff_field(a, b);
                out.end()
            }
            _ => out.difference(a, b),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(unused)]
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
