use fxhash::FxHashSet;
use strum_macros::EnumIter;
use uuid::Uuid;


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[allow(clippy::upper_case_acronyms)]
pub struct XCID (Uuid);

impl XCID {
    pub fn new() -> Self {
        XCID (Uuid::now_v7())
    }
}

#[derive(Debug, Clone)]
pub struct XC {
    pub xc_type: XCType,
    pub id: XCID,
    pub node: usize,
    input_devices: FxHashSet<PortID>,
    output_devices: FxHashSet<PortID>,
    fiber_connection_martrix  : FxHashSet<[PortID; 2]>,          // For FXC and SXC
    waveband_connection_matrix: FxHashSet<(PortID, PortID, WBIndex)>, // For WBXC
}

impl XC {
    pub fn new(node: usize, xc_type: XCType) -> Self {
        Self {
            xc_type,
            id: XCID::new(),
            node,
            input_devices: FxHashSet::default(),
            output_devices: FxHashSet::default(),
            fiber_connection_martrix: FxHashSet::default(),
            waveband_connection_matrix: FxHashSet::default(),
        }
    }

    pub fn get_size(&self) -> usize {
        max(self.input_devices.len(), self.output_devices.len())
    }

    pub fn generate_new_device(&mut self, is_input: bool) -> PortID {
        let id = PortID(Uuid::now_v7());
        if is_input {
            self.input_devices.insert(id);
        } else {
            self.output_devices.insert(id);
        }
        id
    }

    pub fn remove_device(&mut self, target_device_id: PortID, is_input: bool) {
        if is_input {
            if self.has_input_device(&target_device_id) {
                self.input_devices.remove(&target_device_id);
            } else {
                panic!()
            }
        } else if self.has_output_device(&target_device_id) {
            self.output_devices.remove(&target_device_id);
        } else {
            panic!()
        }
    }

    pub fn has_input_device(&self, input_device_id: &PortID) -> bool {
        self.input_devices.contains(input_device_id)
    }
    pub fn has_output_device(&self, output_device_id: &PortID) -> bool {
        self.output_devices.contains(output_device_id)
    }

    pub fn is_input_device_wb_occupied(&self, input_device_id: &PortID, waveband: &WBIndex) -> bool {
        self.waveband_connection_matrix.iter().any(|(i_id, _o_id, wb)| {
            *i_id == *input_device_id && *wb == *waveband
        })
    }
    pub fn is_output_device_wb_occupied(&self, output_device_id: &PortID, waveband: &WBIndex) -> bool {
        self.waveband_connection_matrix.iter().any(|(_i_id, o_id, wb)| {
            *o_id == *output_device_id && *wb == *waveband
        })
    }

    pub fn can_route(&self, input_device_id: &PortID, output_device_id: &PortID) -> bool {
        match self.xc_type {
            XCType::Wxc => true,
            XCType::Added_Wxc => true,
            XCType::Fxc | XCType::Sxc => self.fiber_connection_martrix.contains(&[*input_device_id, *output_device_id]),
            XCType::Wbxc => todo!("WBXC用のテーブルを作成する必要"),
        }
    }

    pub fn can_route_wb(&self, input_device_id: &PortID, output_device_id: &PortID, wb_index: &WBIndex) -> bool {
        
        if self.xc_type != XCType::Wbxc {
            panic!("Invalid XCType");
        }

        self.waveband_connection_matrix.contains(&(*input_device_id, *output_device_id, *wb_index))
    }

    pub fn get_route(&self, input_device_id: &PortID) -> Option<PortID> {
        match self.xc_type {
            XCType::Wxc => {
                // WXC has no strict in routing
                panic!("This call is invalld");
            }
            XCType::Added_Wxc => {
                // WXC has no strict in routing
                panic!("This call is invalld");
            }
            XCType::Fxc | XCType::Sxc => {
                if
                    let Some(connections) = self.fiber_connection_martrix
                        .iter()
                        .find(|p| p[0] == *input_device_id)
                {
                    return Some(connections[1]);
                }
                // eprintln!("Input device with ID: {} has no connection pair", input_device_id);
                None
            }
            XCType::Wbxc => panic!("Invalid call"),
        }
    }

    pub fn has_source(&self, output_port_id: &PortID) -> bool {
        if !self.has_output_device(output_port_id) {
            eprintln!();
            panic!();
        }

        match self.xc_type {
            XCType::Wxc => true,
            XCType::Added_Wxc => true,
            XCType::Wbxc | XCType::Fxc => unimplemented!(),
            XCType::Sxc => {
                self.fiber_connection_martrix.iter().any(|x| x[1] == *output_port_id)
            },
        }
    }

    pub fn has_source_wb(&self, output_port_id: &PortID, wb_index: &WBIndex) -> bool {
        if !self.has_output_device(output_port_id) {
            eprintln!();
            panic!();
        }

        match self.xc_type {
            XCType::Wxc | XCType::Fxc | XCType::Sxc| XCType::Added_Wxc => unimplemented!(),
            XCType::Wbxc => {
                self.waveband_connection_matrix.iter().any(
                    |(_x_in, x_out, x_wb)| *x_out == *output_port_id && *x_wb == *wb_index
                )
            },
        } 
    }
    pub fn has_destination_wb(&self, input_port_id: &PortID, wb_index: &WBIndex) -> bool {
        if !self.has_input_device(input_port_id) {
            eprintln!();
            panic!();
        }

        match self.xc_type {
            XCType::Wxc | XCType::Fxc | XCType::Sxc | XCType::Added_Wxc => unimplemented!(),
            XCType::Wbxc => {
                self.waveband_connection_matrix.iter().any(
                    |(x_in, _x_out, x_wb)| *x_in == *input_port_id && *x_wb == *wb_index
                )
            },
        } 
    }

    pub fn has_destination(&self, input_port_id: &PortID) -> bool {
        if !self.has_input_device(input_port_id) {
            eprintln!();
            panic!();
        }

        match self.xc_type {
            XCType::Wxc => true,
            XCType::Wbxc | XCType::Fxc => unimplemented!(),
            XCType::Sxc => {
                self.fiber_connection_martrix.iter().any(|x| x[0] == *input_port_id)
            },
            XCType::Added_Wxc => true,
        }
    }
    
    pub fn get_route_wbxc_wb(&self, input_device_id: &PortID, waveband: WBIndex) -> Option<PortID> {

        if let Some(connections) = self.waveband_connection_matrix.iter().find(
            |(i_id, _o_id, wb)| *i_id == *input_device_id && *wb == waveband) {
            return Some(connections.1)
        }

        None
    }

    pub fn disconnect_io(&mut self, input_port_id: &PortID, output_port_id: &PortID) -> Result<(), String> {
        if self.xc_type != XCType::Sxc {
            panic!();
        }

        if !(self.has_input_device(input_port_id) && self.has_output_device(output_port_id)) {
            panic!();
        }

        if !self.fiber_connection_martrix.iter().any(|[i_id, o_id]| i_id == input_port_id || o_id == output_port_id) {
            return Err(format!("{input_port_id} or {output_port_id} is not used"));
        }

        self.fiber_connection_martrix.retain(|[i_id, o_id]| !(*i_id == *input_port_id && *o_id == *output_port_id));

        // eprintln!("Disconnected {} and {}", input_port_id, output_port_id);
        Ok(())
    }

    pub fn disconnect_io_wb(&mut self, input_device_id: &PortID, output_device_id: &PortID, waveband: &WBIndex) -> Result<(), String> {
        if self.xc_type != XCType::Wbxc {
            panic!();
        }

        if !(self.has_input_device(input_device_id) && self.has_output_device(output_device_id)) {
            panic!();
        }

        if !self.waveband_connection_matrix.iter().any(|(i_id, o_id, wb)| *wb == *waveband && (i_id == input_device_id || o_id == output_device_id)) {
            return Err(format!("{input_device_id} or {output_device_id} is not used for {waveband:?}"));
        }
        
        self.waveband_connection_matrix.retain(|(i_id, o_id, wb)| !(*wb == *waveband && i_id == input_device_id && o_id == output_device_id));

        Ok(())

    }

    pub fn connect_io(&mut self, input_port_id: &PortID, output_port_id: &PortID) -> Result<(), String> {
        if self.has_input_device(input_port_id) && self.has_output_device(output_port_id) {
            match self.xc_type {
                XCType::Wxc => (), // Nothing to do
                XCType::Fxc | XCType::Sxc => {
                    if
                        self.fiber_connection_martrix.iter().any(|p| p[0] == *input_port_id) ||
                        self.fiber_connection_martrix.iter().any(|p| p[1] == *output_port_id)
                    {
                        // Under used
                        // eprintln!(
                        //     "The device with ID: {} or {} have been used in this XC: {:?}",
                        //     input_device_id,
                        //     output_device_id,
                        //     self
                        // );
                        return Err(format!("Port {} or {} is used.\n {:?}", input_port_id, output_port_id, self))
                    }
                    self.fiber_connection_martrix.insert([*input_port_id, *output_port_id]);
                }
                XCType::Wbxc => unimplemented!("WBXCのconnect_io実装を検討"),
                XCType::Added_Wxc => (),
            }
            // eprintln!("Connected {} and {}", input_port_id, output_port_id);
            Ok(())
        } else {
            // eprintln!(
            //     "This XC: {:?} has no device with ID: {} or {}",
            //     self,
            //     input_device_id,
            //     output_device_id
            // );
            Err("ERROR CODE XX".to_string())
        }
    }

    pub fn connect_io_wb(&mut self, input_device_id: &PortID, output_device_id: &PortID, waveband: &WBIndex) -> Result<(), String> {
        if self.xc_type != XCType::Wbxc {
            panic!();
        }

        if !(self.has_input_device(input_device_id) && self.has_output_device(output_device_id)) {
            return Err("input/output device not found".to_string())
        }

        if self.waveband_connection_matrix.iter().any(
            |(i_id, o_id, wb)| *wb == *waveband && (i_id == input_device_id || o_id == output_device_id)) {
            return Err(format!("{input_device_id} or {output_device_id} is used for {waveband:?}"))
        }

        self.waveband_connection_matrix.insert((*input_device_id, *output_device_id, *waveband));

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter)]
pub enum XCType {
    Wxc = 0,
    Wbxc = 1,
    Fxc = 2,
    Sxc = 3,
    Added_Wxc = 4
}

pub fn xc_type_to_quality_distance(xc_type: XCType) -> usize {
    match xc_type {
        XCType::Wxc => WXC_PORT_Q_DISTANCE,
        XCType::Wbxc => todo!(),
        XCType::Fxc => FXC_PORT_Q_DISTANCE,
        XCType::Sxc => todo!(),
        XCType::Added_Wxc => WXC_PORT_Q_DISTANCE,
    }
}

use std::{cmp::max, fmt};

use crate::{np_core::parameters::{ FXC_PORT_Q_DISTANCE, WXC_PORT_Q_DISTANCE }, WBIndex};

impl fmt::Display for XCType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // フォーマットしたい内容を定義
        match self {
            XCType::Wxc  => write!(f, "Wxc"),
            XCType::Fxc  => write!(f, "Fxc"),
            XCType::Wbxc => write!(f, "Wbxc"),
            XCType::Sxc  => write!(f, "Sxc"),
            XCType::Added_Wxc  => write!(f, "Added_Wxc"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PortID (Uuid);
impl PortID {
    pub(crate) fn nil() -> PortID {
        PortID(Uuid::nil())
    }
}

impl fmt::Display for PortID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}