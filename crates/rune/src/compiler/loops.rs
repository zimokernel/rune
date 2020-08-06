use crate::ast;
use crate::compiler::{NeedsValue, Result};
use crate::error::CompileError;
use crate::source::Source;
use stk::unit::{Label, Span};

#[must_use]
pub(super) struct LoopGuard(usize);

/// Loops we are inside.
#[derive(Clone, Copy)]
pub(super) struct Loop {
    /// The optional label of the loop.
    pub(super) label: Option<ast::Label>,
    /// The end label of the loop.
    pub(super) break_label: Label,
    /// The number of variables observed at the start of the loop.
    pub(super) total_var_count: usize,
    /// If the loop needs a value.
    pub(super) needs_value: NeedsValue,
}

pub(super) struct Loops {
    loops: Vec<Loop>,
}

impl Loops {
    /// Construct a new collection of loops.
    pub(super) fn new() -> Self {
        Self { loops: vec![] }
    }

    /// Get the last loop context.
    pub(super) fn last(&self) -> Option<Loop> {
        self.loops.last().copied()
    }

    /// Push loop information.
    pub(super) fn push(&mut self, l: Loop) -> LoopGuard {
        self.loops.push(l);
        LoopGuard(self.loops.len())
    }

    pub(super) fn pop(&mut self, span: Span, guard: LoopGuard) -> Result<()> {
        let LoopGuard(loop_count) = guard;

        if loop_count != self.loops.len() {
            return Err(CompileError::internal(
                "loop: loop count mismatch on return",
                span,
            ));
        }

        if self.loops.pop().is_none() {
            return Err(CompileError::internal("loop: missing parent loop", span));
        }

        Ok(())
    }

    /// Find the loop with the matching label.
    pub(super) fn walk_until_label(
        &self,
        source: Source<'_>,
        expected: ast::Label,
    ) -> Result<Loop> {
        use crate::traits::Resolve as _;

        let span = expected.span();
        let expected = expected.resolve(source)?;

        for l in self.loops.iter().rev() {
            let label = match l.label {
                Some(label) => label,
                None => continue,
            };

            let label = label.resolve(source)?;

            if expected == label {
                return Ok(*l);
            }
        }

        Err(CompileError::MissingLabel { span })
    }
}