use std::collections::HashMap;

use block_ir::BlockId;

/// Maps every statement block to the 1-indexed line of generated Lua where
/// its code begins, and back.
///
/// This is the runtime contract's "source map" (`docs/technical-design.md`):
/// the `runtime` crate resolves a pause position (a Lua line) back to a
/// `BlockId` to report/highlight, and maps a breakpoint (a `BlockId`) to a
/// Lua line before it reaches `runtime`. Only statements are mapped —
/// there's no granularity below "which statement is executing" in the
/// MVP's step model, so expressions don't get their own entries.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SourceMap {
    block_to_line: HashMap<BlockId, u32>,
    line_to_block: HashMap<u32, BlockId>,
}

impl SourceMap {
    /// An empty source map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Records that `block`'s generated code begins at `line`.
    pub(crate) fn record(&mut self, block: BlockId, line: u32) {
        self.block_to_line.insert(block, line);
        self.line_to_block.insert(line, block);
    }

    /// The line `block`'s generated code begins at, if it was mapped.
    pub fn line_for(&self, block: BlockId) -> Option<u32> {
        self.block_to_line.get(&block).copied()
    }

    /// The block whose generated code begins at `line`, if any.
    pub fn block_at(&self, line: u32) -> Option<BlockId> {
        self.line_to_block.get(&line).copied()
    }

    /// How many blocks are mapped.
    pub fn len(&self) -> usize {
        self.block_to_line.len()
    }

    /// Whether no blocks are mapped.
    pub fn is_empty(&self) -> bool {
        self.block_to_line.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_both_directions() {
        let mut map = SourceMap::new();
        let id = BlockId::from_raw(7);
        map.record(id, 3);
        assert_eq!(map.line_for(id), Some(3));
        assert_eq!(map.block_at(3), Some(id));
    }

    #[test]
    fn unmapped_lookups_are_none() {
        let map = SourceMap::new();
        assert_eq!(map.line_for(BlockId::from_raw(0)), None);
        assert_eq!(map.block_at(1), None);
    }
}
