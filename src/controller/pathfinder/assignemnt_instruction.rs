use crate::network::{CoreIndex, FiberID};

pub struct AssignmentInstruction {
    pub fiber_ids: Vec<FiberID>,
    pub core_indices: Vec<CoreIndex>,
    pub slot_head: Vec<usize>,
    pub slot_width: usize
}