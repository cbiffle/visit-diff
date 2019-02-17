use super::*;

/// Diff boxes by dereferencing.
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

/// Diff Cow by dereferencing.
impl<'a, T> Diff for std::borrow::Cow<'a, T>
where
    T: Clone + Diff,
{
    fn diff<D>(a: &Self, b: &Self, out: D) -> Result<D::Ok, D::Err>
    where
        D: Differ,
    {
        Diff::diff(&**a, &**b, out)
    }
}

impl_diff_partial_eq!(String);
impl_diff_partial_eq!(std::io::ErrorKind);
impl_diff_partial_eq!(std::io::SeekFrom);
impl_diff_partial_eq!(std::net::Ipv4Addr);
impl_diff_partial_eq!(std::net::Ipv6Addr);
impl_diff_partial_eq!(std::net::SocketAddrV4);
impl_diff_partial_eq!(std::net::SocketAddrV6);
impl_diff_partial_eq!(std::net::IpAddr);
impl_diff_partial_eq!(std::net::SocketAddr);

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

impl<V> Diff for std::collections::VecDeque<V>
where
    V: Diff,
{
    fn diff<D>(a: &Self, b: &Self, out: D) -> Result<D::Ok, D::Err>
    where
        D: Differ,
    {
        let mut out = out.begin_seq();
        out.diff_elements(a.iter(), b.iter());
        out.end()
    }
}

impl<V> Diff for std::collections::LinkedList<V>
where
    V: Diff,
{
    fn diff<D>(a: &Self, b: &Self, out: D) -> Result<D::Ok, D::Err>
    where
        D: Differ,
    {
        let mut out = out.begin_seq();
        out.diff_elements(a.iter(), b.iter());
        out.end()
    }
}

impl<K, V> Diff for std::collections::BTreeMap<K, V>
where
    K: Ord + Debug,
    V: Diff,
{
    fn diff<D>(a: &Self, b: &Self, out: D) -> Result<D::Ok, D::Err>
    where
        D: Differ,
    {
        use itertools::{EitherOrBoth, Itertools};

        let mut out = out.begin_map();

        for ab in a.iter().merge_join_by(b, |(i, _), (j, _)| i.cmp(j)) {
            match ab {
                EitherOrBoth::Left((k, v)) => out.only_in_left(k, v),
                EitherOrBoth::Right((k, v)) => out.only_in_right(k, v),
                EitherOrBoth::Both((k, a), (_, b)) => out.diff_entry(k, a, b),
            }
        }

        out.end()
    }
}

impl<K, V> Diff for std::collections::HashMap<K, V>
where
    K: Eq + std::hash::Hash + Debug,
    V: Diff,
{
    fn diff<D>(a: &Self, b: &Self, out: D) -> Result<D::Ok, D::Err>
    where
        D: Differ,
    {
        let mut out = out.begin_map();

        for (k, va) in a {
            if let Some(vb) = b.get(k) {
                out.diff_entry(k, va, vb)
            } else {
                out.only_in_left(k, va)
            }
        }

        for (k, vb) in b {
            if !a.contains_key(k) {
                out.only_in_right(k, vb)
            }
        }

        out.end()
    }
}

impl<K> Diff for std::collections::BTreeSet<K>
where
    K: Ord + Diff,
{
    fn diff<D>(a: &Self, b: &Self, out: D) -> Result<D::Ok, D::Err>
    where
        D: Differ,
    {
        use itertools::{EitherOrBoth, Itertools};

        let mut out = out.begin_set();

        for ab in a.iter().merge_join_by(b, |i, j| i.cmp(j)) {
            match ab {
                EitherOrBoth::Left(a) => out.only_in_left(a),
                EitherOrBoth::Right(a) => out.only_in_right(a),
                EitherOrBoth::Both(a, b) => out.diff_equal(a, b),
            }
        }

        out.end()
    }
}

impl<K> Diff for std::collections::HashSet<K>
where
    K: std::hash::Hash + Eq + Diff,
{
    fn diff<D>(a: &Self, b: &Self, out: D) -> Result<D::Ok, D::Err>
    where
        D: Differ,
    {
        let mut out = out.begin_set();

        for e in a.intersection(b) {
            out.diff_equal(e, b.get(e).unwrap());
        }

        for e in a.difference(b) {
            out.only_in_left(e);
        }

        for e in b.difference(a) {
            out.only_in_right(e);
        }

        out.end()
    }
}
