use crate::sbx_specs::Version;

pub const VERSION: Version = Version::V17;

pub const RS_DATA: usize = 10;

pub const RS_PARITY: usize = 2;

pub const BURST: usize = 12;

pub const DATA_PAR_BURST: (usize, usize, usize) = (RS_DATA, RS_PARITY, BURST);
