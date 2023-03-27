use cranelift::codegen::cursor::{Cursor, FuncCursor};
use cranelift::prelude::FunctionBuilder;

pub trait FunctionBuilderExt {
    /// Checks if the current block has a terminator as its last instruction.
    fn is_filled(&mut self) -> bool;

    /// Checks if the current block has no instructions.
    fn is_pristine(&mut self) -> bool;
}

impl<'a> FunctionBuilderExt for FunctionBuilder<'a> {
    /// Checks if the current block in the builder has a terminator as its last instruction.
    fn is_filled(&mut self) -> bool {
        let block = match self.current_block() {
            Some(block) => block,
            None => return false,
        };

        let cur = FuncCursor::new(&mut self.func);
        if !cur.layout().is_block_inserted(block) {
            return false;
        }
        let mut cur = cur.at_bottom(block);

        let last_inst = match cur.prev_inst() {
            Some(inst) => inst,
            // If we don't have a last instruction then the block is empty
            None => return false,
        };
        let opcode = self.func.dfg.insts[last_inst].opcode();
        opcode.is_terminator()
    }

    /// Checks if the current block in the builder has no instructions.
    fn is_pristine(&mut self) -> bool {
        let block = match self.current_block() {
            Some(block) => block,
            None => return false,
        };

        let cur = FuncCursor::new(&mut self.func);
        if !cur.layout().is_block_inserted(block) {
            return false;
        }
        let mut cur = cur.at_bottom(block);

        cur.prev_inst().is_none()
    }
}
