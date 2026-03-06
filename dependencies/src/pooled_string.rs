use std::{
    borrow::Borrow,
    cmp::Ordering,
    collections::HashSet,
    convert::Infallible,
    fmt::{self, Display, Formatter},
    ops::Deref,
    str::FromStr,
    sync::{LazyLock, Mutex},
};

/// The set of allocated strings.
static REGISTRY: LazyLock<Mutex<HashSet<PooledString>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

/// A shared string type that guarantees that two equal strings will refer to the same allocated
/// memory. Strings are not deallocated while the program is running.
#[expect(
    clippy::derived_hash_with_manual_eq,
    reason = "The manual implementation of PartialEq uses PooledString's invariant to reduce comparison to constant time without changing the result"
)]
#[derive(Clone, Copy, Debug, Eq, Hash)]
pub struct PooledString(&'static str);

impl Display for PooledString {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&'static str> for PooledString {
    fn from(s: &'static str) -> Self {
        let mut registry_guard = REGISTRY.lock().unwrap();
        let registry = &mut *registry_guard;
        registry.get(s).copied().unwrap_or_else(|| {
            let ret = Self(s);
            registry.insert(ret);
            ret
        })
    }
}

impl From<String> for PooledString {
    fn from(s: String) -> Self {
        let mut registry_guard = REGISTRY.lock().unwrap();
        let registry = &mut *registry_guard;
        registry.get(&*s).copied().unwrap_or_else(|| {
            let ret = Self(&*s.leak());
            registry.insert(ret);
            ret
        })
    }
}

impl FromStr for PooledString {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut registry_guard = REGISTRY.lock().unwrap();
        let registry = &mut *registry_guard;
        Ok(registry.get(s).copied().unwrap_or_else(|| {
            let ret = Self(&*s.to_string().leak());
            registry.insert(ret);
            ret
        }))
    }
}

impl Borrow<str> for PooledString {
    fn borrow(&self) -> &str {
        self.0
    }
}

impl Deref for PooledString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl PartialEq for PooledString {
    fn eq(&self, other: &Self) -> bool {
        // Since we guarantee that two `PooledString`s that wrap equal strings wrap *the same*
        // string, testing for equality between `PooledString`s can be reduced to a check that they
        // have the same backing memory.
        // We need to compare lengths in addition to starting position because we do not clone the
        // backing memory when a `PooledString` is created via `From::<&'static str>::from`.
        self.0.as_ptr() == other.0.as_ptr() && self.0.len() == other.0.len()
    }
}

impl PartialEq<str> for PooledString {
    fn eq(&self, other: &str) -> bool {
        matches!(self.partial_cmp(other), Some(Ordering::Equal))
    }
}

impl PartialEq<PooledString> for str {
    fn eq(&self, other: &PooledString) -> bool {
        matches!(self.partial_cmp(other), Some(Ordering::Equal))
    }
}

impl PartialOrd<str> for PooledString {
    fn partial_cmp(&self, other: &str) -> Option<std::cmp::Ordering> {
        let this = &**self;
        Some(this.cmp(other))
    }
}

impl PartialOrd<PooledString> for str {
    fn partial_cmp(&self, other: &PooledString) -> Option<std::cmp::Ordering> {
        let other = &**other;
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eq() {
        const S: &str = "abc";
        let ps = PooledString::from(S);
        let ps1 = PooledString::from(&S[..1]);
        assert_ne!(ps, ps1);
        let ps2 = "abc".parse::<PooledString>().unwrap();
        assert_eq!(ps, ps2);
    }
}
