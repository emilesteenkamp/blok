/// Stable identifier for a block instance in the tree.
///
/// Carried through `lua-transpiler`'s source map so the runtime can report
/// pause/highlight positions and the editor can map breakpoints back to a
/// block, per the runtime contract in `docs/technical-design.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlockId(u64);

impl BlockId {
    /// Wraps a raw id value.
    ///
    /// Prefer [`BlockIdGenerator::next_id`] when minting *new* ids for a
    /// tree under construction; use this when reconstructing a `BlockId`
    /// from a stored or previously-issued raw value (e.g. deserializing a
    /// saved document, or mapping a runtime pause position back to an id).
    pub fn from_raw(raw: u64) -> Self {
        BlockId(raw)
    }

    /// The underlying id value, e.g. for persistence or for passing across
    /// the `runtime-protocol` boundary.
    pub fn raw(self) -> u64 {
        self.0
    }
}

/// Issues unique, increasing [`BlockId`]s. The editor owns one of these for
/// the lifetime of a document.
///
/// # Examples
///
/// ```
/// use block_ir::BlockIdGenerator;
///
/// let mut ids = BlockIdGenerator::new();
/// let a = ids.next_id();
/// let b = ids.next_id();
/// assert_ne!(a, b);
/// ```
#[derive(Debug, Default)]
pub struct BlockIdGenerator {
    next: u64,
}

impl BlockIdGenerator {
    /// Creates a generator that issues ids starting from `0`.
    pub fn new() -> Self {
        Self { next: 0 }
    }

    /// Mints the next unused id.
    ///
    /// Ids are issued in increasing order starting from `0` and never
    /// reused, so every id returned by a given generator is unique.
    pub fn next_id(&mut self) -> BlockId {
        let id = BlockId(self.next);
        self.next += 1;
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generator_issues_unique_increasing_ids() {
        let mut gen = BlockIdGenerator::new();
        let a = gen.next_id();
        let b = gen.next_id();
        let c = gen.next_id();
        assert_ne!(a, b);
        assert_ne!(b, c);
        assert!(a.raw() < b.raw());
        assert!(b.raw() < c.raw());
    }
}
