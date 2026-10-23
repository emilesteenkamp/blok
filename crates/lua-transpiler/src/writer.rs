//! A line-tracking buffer for emitting indented Lua source.

const INDENT_UNIT: &str = "    ";

/// Accumulates generated Lua source line by line, tracking the 1-indexed
/// line number each write lands on so callers can record
/// [`crate::SourceMap`] entries as they go.
pub(crate) struct Writer {
    buf: String,
    line: u32,
    indent: usize,
}

impl Writer {
    pub(crate) fn new() -> Self {
        Self {
            buf: String::new(),
            line: 1,
            indent: 0,
        }
    }

    /// The line the next [`Writer::writeln`] call will write to.
    pub(crate) fn current_line(&self) -> u32 {
        self.line
    }

    /// Writes one line, indented to the current nesting level.
    pub(crate) fn writeln(&mut self, content: &str) {
        for _ in 0..self.indent {
            self.buf.push_str(INDENT_UNIT);
        }
        self.buf.push_str(content);
        self.buf.push('\n');
        self.line += 1;
    }

    /// Increases indentation for subsequent lines, for the duration of `f`.
    pub(crate) fn indented<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        self.indent += 1;
        let result = f(self);
        self.indent -= 1;
        result
    }

    /// Consumes the writer, returning the accumulated source.
    pub(crate) fn finish(self) -> String {
        self.buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracks_line_numbers_across_writes() {
        let mut w = Writer::new();
        assert_eq!(w.current_line(), 1);
        w.writeln("local a = 1");
        assert_eq!(w.current_line(), 2);
        w.writeln("local b = 2");
        assert_eq!(w.current_line(), 3);
    }

    #[test]
    fn indents_nested_writes() {
        let mut w = Writer::new();
        w.writeln("if true then");
        w.indented(|w| w.writeln("local a = 1"));
        w.writeln("end");
        assert_eq!(w.finish(), "if true then\n    local a = 1\nend\n");
    }
}
