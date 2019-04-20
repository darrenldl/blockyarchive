use std::sync::{Arc};

use crate::progress_report::*;

use smallvec::SmallVec;

use crate::misc_utils::{RangeEnd};

use crate::sbx_specs::{SBX_FILE_UID_LEN, SBX_LARGEST_BLOCK_SIZE, SBX_SCAN_BLOCK_SIZE};

use crate::sbx_block::{Block};
use crate::sbx_block::Metadata;

use crate::json_printer::{BracketType, JSONPrinter};

pub struct Param {
    in_file: String,
    new_meta: SmallVec<[Metadata; 8]>,
    from_pos: Option<u64>,
    to_pos: Option<RangeEnd<u64>>,
    force_misalign: bool,
    json_printer: Arc<JSONPrinter>,
    only_pick_uid: Option<[u8; SBX_FILE_UID_LEN]>,
    pr_verbosity_level: PRVerbosityLevel,
}

impl Param {
    pub fn new(
        in_file: &str,
        new_meta: SmallVec<[Metadata; 8]>,
        from_pos: Option<u64>,
        to_pos: Option<RangeEnd<u64>>,
        force_misalign: bool,
        json_printer: &Arc<JSONPrinter>,
        only_pick_uid: Option<[u8; SBX_FILE_UID_LEN]>,
        pr_verbosity_level: PRVerbosityLevel,
    ) -> Param {
        Param {
            in_file: String::from(in_file),
            new_meta,
            from_pos,
            to_pos,
            force_misalign,
            json_printer: Arc::clone(json_printer),
            only_pick_uid: match only_pick_uid {
                None => None,
                Some(x) => Some(x.clone()),
            },
            pr_verbosity_level,
        }
    }
}

fn update_metas(block: &mut Block, metas: &[Metadata]) {
    assert!(block.is_meta());
    block.update_metas(metas).unwrap();
}
