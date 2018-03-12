mod encoder;
pub use self::encoder::RSEncoder;

use super::smallvec::SmallVec;

use super::sbx_block::BlockType;
use super::sbx_specs::Version;

use super::sbx_specs::ver_to_block_size;

use std::fmt;

mod repairer;
pub use self::repairer::RSRepairer;
pub use self::repairer::RSRepairStats;

mod tests;

use super::Error;
use super::ErrorKind;

#[must_use]
pub enum RSCodecState {
    Ready,
    NotReady
}
