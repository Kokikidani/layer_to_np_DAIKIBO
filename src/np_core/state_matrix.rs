use std::{fmt::Display, ops::{ BitAnd, BitAndAssign, BitOr, BitOrAssign, Index, IndexMut, Range }};

use super::{parameters::SLOT, WBIndex};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StateMatrix([bool; SLOT]);
impl BitAnd for StateMatrix {
    type Output = StateMatrix;

    fn bitand(self, rhs: Self) -> Self::Output {
        let mut output = [false; SLOT];

        for (index, (&self_s, &rhs_s)) in self.0.iter().zip(rhs.0.iter()).enumerate() {
            output[index] = self_s & rhs_s;
        }

        StateMatrix(output)
    }
}
impl BitAndAssign for StateMatrix {

    fn bitand_assign(&mut self, rhs: Self) {
        for (self_s, &rhs_s) in self.0.iter_mut().zip(rhs.0.iter()) {
            *self_s &= rhs_s;
        }
    }
}

impl BitOr for StateMatrix {
    type Output = StateMatrix;

    fn bitor(self, rhs: Self) -> Self::Output {
        let mut output = [false; SLOT];

        for (index, (&self_s, &rhs_s)) in self.0.iter().zip(rhs.0.iter()).enumerate() {
            output[index] = self_s | rhs_s;
        }

        StateMatrix(output)
    }
}
impl BitOrAssign for StateMatrix {
    fn bitor_assign(&mut self, rhs: Self) {
        for (self_s, &rhs_s) in self.0.iter_mut().zip(rhs.0.iter()) {
            *self_s |= rhs_s;
        }
    }
}

// Implementing Index and IndexMut traits
impl Index<usize> for StateMatrix {
    type Output = bool;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for StateMatrix {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl Index<Range<usize>> for StateMatrix {
    type Output = [bool];

    fn index(&self, index: Range<usize>) -> &Self::Output {
        &self.0[index]
    }
}
impl IndexMut<Range<usize>> for StateMatrix {
    fn index_mut(&mut self, index: Range<usize>) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl StateMatrix {
    pub fn new() -> StateMatrix {
        Self([false; SLOT])
    }

    pub fn new_fulfilled() -> StateMatrix {
        let mut output = Self::new();
        for s_index in 0..output.0.len() {
           output[s_index] = true;
        }
        output
    }

    pub fn are_slots_empty(&self, slot: usize, width: usize) -> bool {
        for s in &self.0[slot..slot+width] {
            if *s {
                return false
            }
        }

        true
    }

    pub fn are_slots_full(&self, slot: usize, width: usize) -> bool {
        for s in &self.0[slot..slot+width] {
            if !*s {
                return false
            }
        }

        true
    }

    pub fn is_empty(&self) -> bool {
        self.0.iter().all(|x| !*x)
    }

    pub fn has_empty_contiguous_slots(&self, size: usize) -> bool {
        self.get_empty_contiguous_slots(size).is_some()
    }

    pub fn iter(&self) -> std::slice::Iter<bool>  {
        self.0.iter()
    }

    pub fn get_empty_contiguous_slots(&self, size: usize) -> Option<usize> {
        let mut target_state_matrix = *self;
        let mut state_matrix_for_shift = *self;

        for _ in 0..size - 1 {
            state_matrix_for_shift.r_shift();
            target_state_matrix |= state_matrix_for_shift;
        }

        target_state_matrix.0.iter().position(|x| !*x)
    }

    fn r_shift(&mut self) {
        let mut prev_s = true;
        for s in self.0.iter_mut().rev() {
            std::mem::swap(&mut *s, &mut prev_s);
        }
    }

    pub fn get_raw(self) -> [bool; SLOT] {
        self.0
    }

    pub fn apply_witout_wb_filter(&mut self, wb: &WBIndex) {
        for (idx, s) in self.0.iter_mut().enumerate() {
            if !wb.includes(idx) {
                *s = true;
            }
        }
    }
}

impl Default for StateMatrix {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for StateMatrix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for element in self.0 {
            if element {
                write!(f, "█")?;
            } else {
                write!(f, "▏")?;
            }
        }
        Ok(())
    }
}