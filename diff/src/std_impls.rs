use super::*;

/// Diff boxes by dereferencing.
#[cfg(feature = "std")]
impl<T> Diff for Box<T>
where
    T: Diff,
{
    fn diff<D>(a: &Self, b: &Self, out: D) -> Result<D::Ok, D::Err>
    where
        D: Differ,
    {
        Diff::diff(&**a, &**b, out)
    }
}

/// Diff Rcs by dereferencing.
#[cfg(feature = "std")]
impl<T> Diff for std::rc::Rc<T>
where
    T: Diff,
{
    fn diff<D>(a: &Self, b: &Self, out: D) -> Result<D::Ok, D::Err>
    where
        D: Differ,
    {
        Diff::diff(&**a, &**b, out)
    }
}

/// Diff Arcs by dereferencing.
#[cfg(feature = "std")]
impl<T> Diff for std::sync::Arc<T>
where
    T: Diff,
{
    fn diff<D>(a: &Self, b: &Self, out: D) -> Result<D::Ok, D::Err>
    where
        D: Differ,
    {
        Diff::diff(&**a, &**b, out)
    }
}

#[cfg(feature = "std")]
impl_diff_partial_eq!(String);

#[cfg(feature = "std")]
impl<V> Diff for Vec<V>
where
    V: Diff,
{
    fn diff<D>(a: &Self, b: &Self, out: D) -> Result<D::Ok, D::Err>
    where
        D: Differ,
    {
        Diff::diff(&a.as_slice(), &b.as_slice(), out)
    }
}

#[cfg(feature = "std")]
impl<K, V> Diff for std::collections::BTreeMap<K, V>
where
    K: Ord + Debug,
    V: Diff,
{
    fn diff<D>(a: &Self, b: &Self, out: D) -> Result<D::Ok, D::Err>
    where
        D: Differ,
    {
        use core::cmp::Ordering;

        let mut out = out.begin_map();

        let mut akeys = a.keys().peekable();
        let mut bkeys = b.keys().peekable();

        while let (Some(ka), Some(kb)) = (akeys.peek(), bkeys.peek()) {
            match ka.cmp(kb) {
                Ordering::Less => {
                    out.only_in_left(ka, &a[ka]);
                    akeys.next();
                }
                Ordering::Equal => {
                    out.diff_entry(ka, &a[ka], &b[kb]);
                    akeys.next();
                    bkeys.next();
                }
                Ordering::Greater => {
                    out.only_in_right(kb, &b[kb]);
                    bkeys.next();
                }
            }
        }

        for k in akeys {
            out.only_in_left(k, &a[k])
        }
        for k in bkeys {
            out.only_in_right(k, &b[k])
        }

        out.end()
    }
}

#[cfg(feature = "std")]
impl<K> Diff for std::collections::BTreeSet<K>
where
    K: Ord + Diff,
{
    fn diff<D>(a: &Self, b: &Self, out: D) -> Result<D::Ok, D::Err>
    where
        D: Differ,
    {
        use core::cmp::Ordering;

        let mut out = out.begin_set();

        let mut akeys = a.iter().peekable();
        let mut bkeys = b.iter().peekable();

        while let (Some(ka), Some(kb)) = (akeys.peek(), bkeys.peek()) {
            match ka.cmp(kb) {
                Ordering::Less => {
                    out.only_in_left(ka);
                    akeys.next();
                }
                Ordering::Equal => {
                    out.diff_equal(ka, kb);
                    akeys.next();
                    bkeys.next();
                }
                Ordering::Greater => {
                    out.only_in_right(kb);
                    bkeys.next();
                }
            }
        }

        for k in akeys {
            out.only_in_left(k)
        }
        for k in bkeys {
            out.only_in_right(k)
        }

        out.end()
    }
}
