use core::fmt;

use fxhash::FxHashSet;
use uuid::Uuid;

use crate::{ np_core::{parameters::CORE_FACTOR, StateMatrix}, utils::generate_uuid, Edge, SLOT };

use super::{xc::PortID, XCType};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FiberID (Uuid);
// impl FiberID {
//     pub(crate) fn nil() -> Self {
//         FiberID(Uuid::nil())
//     }
// }
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CoreIndex (usize);
impl CoreIndex {
    pub fn new(value: usize) -> Self {
        Self (value)
    }
    pub fn index(&self) -> usize {
        self.0
    }
    pub fn iter() -> Vec<Self> {
        (0..CORE_FACTOR).map(CoreIndex::new).collect()
    }
}
impl From<CoreIndex> for usize {
    fn from(val: CoreIndex) -> Self {
        val.0
    }
}
impl From<usize> for CoreIndex {
    fn from(value: usize) -> Self {
        CoreIndex(value)
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FiberType {
    Scf = 0,
    Mcf = 1
}

#[derive(Debug, Clone)]
pub struct Fiber {
    pub fiber_id: FiberID,
    pub edge: Edge,
    pub state_matrixes: Vec<StateMatrix>,
    pub assigned_demand_ids: FxHashSet<usize>,
    pub occupancy: usize,
    pub residual: usize,
    pub src_port_ids: Vec<PortID>,
    pub dst_port_ids: Vec<PortID>,
    pub sd_xc_type: [XCType; 2],
    pub distance: usize,
    pub fiber_type: FiberType
}

impl fmt::Display for FiberID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}


impl Fiber {
    /// # ONLY MAKE FIBER STURCT!!!
    /// # THIS FUNCTION DOES NOT MANAGE REGISTRATION TO NETWORK!!
    pub fn new_scf(edge: &Edge, src_port_id: PortID, dst_port_id: PortID, sd_xc_type: [XCType; 2]) -> Fiber {
        Fiber {
            state_matrixes: vec![StateMatrix::new()],
            edge: *edge,
            assigned_demand_ids: FxHashSet::default(),
            occupancy: 0,
            residual: SLOT,
            fiber_id: FiberID(generate_uuid()),
            src_port_ids: vec![src_port_id],
            dst_port_ids: vec![dst_port_id],
            sd_xc_type,
            distance: 0,
            fiber_type: FiberType::Scf,
        }
    }

    pub fn new_mcf(edge: &Edge, src_port_ids: Vec<PortID>, dst_port_ids: Vec<PortID>, sd_xc_type: [XCType; 2]) -> Fiber {
        Fiber {
            state_matrixes: vec![StateMatrix::new(); CORE_FACTOR],
            edge: *edge,
            assigned_demand_ids: FxHashSet::default(),
            occupancy: 0,
            residual: SLOT*CORE_FACTOR,
            fiber_id: FiberID(generate_uuid()),
            src_port_ids,
            dst_port_ids,
            sd_xc_type,
            distance: 0,
            fiber_type: FiberType::Mcf,
        }
    }

     /// 使用中スロット数
     pub fn count_used_slots(&self) -> usize {
        self.occupancy
    }

    /// 総スロット数（使用中 + 空き）
    pub fn total_slots(&self) -> usize {
        self.occupancy + self.residual
    }

    pub fn assign(&mut self, slot: usize, width: usize, core_index: &CoreIndex, demand_id: usize) {
        if self.assigned_demand_ids.contains(&demand_id) {
            eprintln!("{}", demand_id);
            panic!("This demand path is already assigend to the fiber");
        }

        // StateMatrix checking
        if !self.state_matrixes[core_index.index()].are_slots_empty(slot, width) {
            println!("{:?}", self.state_matrixes);
            panic!("Slot {} to {} is occupied. Use another slots.", slot, slot+width-1);
        }

        for s in &mut self.state_matrixes[core_index.index()][slot..slot+width] {
            *s = true;
        }
        self.assigned_demand_ids.insert(demand_id);
        self.occupancy += width;
        self.residual -= width;
    }

    pub fn delete(&mut self, slot: usize, width: usize, core_index: &CoreIndex, demand_id: usize) {
        if !self.assigned_demand_ids.contains(&demand_id) {
            eprintln!("{}", demand_id);
            panic!("This demand path is not assigend to this fiber");
        }

        // StateMatrix checking
        if !self.state_matrixes[core_index.index()].are_slots_full(slot, width) {
            panic!("Slot {} to {} is empty. Something went wrong.", slot, slot+width-1);
        }

        for s in &mut self.state_matrixes[core_index.index()][slot..slot+width] {
            *s = false;
        }
        self.assigned_demand_ids.remove(&demand_id);
        self.occupancy -= width;
        self.residual += width;
    }

    pub fn is_full(&self) -> bool {
        for core_index_as_usize in 0..self.get_core_num() {
            if self.state_matrixes[core_index_as_usize].has_empty_contiguous_slots(1) {
                return false
            }
        }

        true
    }

    pub fn get_core_num(&self) -> usize {
        self.state_matrixes.len()
    }
}
