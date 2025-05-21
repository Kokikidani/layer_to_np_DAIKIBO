use super::parameters::{SLOT, WAVEBAND_COUNT};

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct WBIndex (usize);

impl WBIndex {
    pub fn new(waveband_index: usize) -> WBIndex {
        WBIndex(waveband_index)
    }

    pub fn includes(&self, slot: usize) -> bool {
        let waveband_index = self.index();

        waveband_index * SLOT/WAVEBAND_COUNT <= slot && slot < (waveband_index + 1) * SLOT/WAVEBAND_COUNT
    }

    pub fn from_wavelength(slot: usize) -> Self {
        let table = WBIndex::get_wb_table();

        let waveband_index = table.iter().enumerate().find(|(_row_index, row)| {
            matches!(row.iter().position(|&x| x == slot), Some(_col_index))
        }).map(|(row_index, _row)| row_index).unwrap();

        WBIndex(waveband_index)
    }

    fn get_wb_table() -> [[usize; SLOT/WAVEBAND_COUNT]; WAVEBAND_COUNT] {
        let mut output: [[usize; SLOT/WAVEBAND_COUNT]; WAVEBAND_COUNT] = [[0; SLOT/WAVEBAND_COUNT]; WAVEBAND_COUNT];
        for (waveband_index, row) in output.iter_mut().enumerate() {
            for (i, elem) in row.iter_mut().enumerate() {
                *elem = waveband_index * SLOT/WAVEBAND_COUNT + i;
            }
        }
        output
    }
    
    pub fn iter() -> impl DoubleEndedIterator<Item = WBIndex> {
        let mut output: Vec<WBIndex> = Vec::with_capacity(WAVEBAND_COUNT);
        for i in 0..WAVEBAND_COUNT {
            output.push(WBIndex(i));
        }
        output.into_iter()
    }

    pub fn index(&self) -> usize {
        self.0
    }
}

