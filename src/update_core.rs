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
    dry_run: bool,
    metas_to_add: SmallVec<[Metadata; 8]>,
    force_misalign: bool,
    json_printer: Arc<JSONPrinter>,
    update_all: bool,
    pr_verbosity_level: PRVerbosityLevel,
}

impl Param {
    pub fn new(
        in_file: &str,
        dry_run: bool,
        metas_to_add: SmallVec<[Metadata; 8]>,
        from_pos: Option<u64>,
        to_pos: Option<RangeEnd<u64>>,
        force_misalign: bool,
        json_printer: &Arc<JSONPrinter>,
        only_pick_uid: Option<[u8; SBX_FILE_UID_LEN]>,
        update_all: bool,
        pr_verbosity_level: PRVerbosityLevel,
    ) -> Param {
        Param {
            in_file: String::from(in_file),
            dry_run,
            metas_to_add,
            from_pos,
            to_pos,
            force_misalign,
            json_printer: Arc::clone(json_printer),
            only_pick_uid: match only_pick_uid {
                None => None,
                Some(x) => Some(x.clone()),
            },
            update_all,
            pr_verbosity_level,
        }
    }
}

#[derive(Clone)]
pub struct Stats {
    pub meta_blocks_updated: u64,
    pub meta_blocks_failed: u64,
    total_meta_blocks: u64,
    start_time: f64,
    end_time: f64,
    json_printer: Arc<JSONPrinter>,
}

impl Stats {
    pub fn new(ref_block: &Block, data_par_burst: Option<(usize, usize, usize)>, json_printer: &Arc<JSONPrinter>) -> Stats {
        use crate::file_utils::from_container_size::calc_total_block_count;
        let total_meta_blocks = sbx_block::calc_meta_block_all_write_pos_s(ref_block.get_version(), data_par_burst);
    }
}

fn update_metas(block: &mut Block, metas: &[Metadata]) {
    assert!(block.is_meta());
    block.update_metas(metas).unwrap();
}

pub fn update_file(param: &Param) -> Result<Stats, Error> {
    let ctrlc_stop_flag = setup_ctrlc_handler(param.json_printer.json_enabled());

    let json_printer = &param.json_printer;

    let (ref_block_pos, mut ref_block) = get_ref_block!( no_force_misalign =>
                                                         param,
                                                         None,
                                                         None,
                                                         json_printer,
                                                         RefBlockChoice::MustBe(BlockType::Meta),
                                                         ctrlc_stop_flag
    );

    let version = ref_block.get_version();

    let block_size = ver_to_block_size(version);

    let pred = block_pred_same_ver_uid!(ref_block);

    let burst = get_burst_or_guess!(no_offset => param, ref_block_pos, ref_block);

    let data_par_burst = Some((
        get_RSD_from_ref_block!(ref_block_pos, ref_block, "repair"),
        get_RSP_from_ref_block!(ref_block_pos, ref_block, "repair"),
        burst,
    ));

    let total_block_count = {
        use crate::file_utils::from_orig_file_size::calc_total_block_count_exc_burst_gaps;
        match ref_block.get_FSZ().unwrap() {
            Some(x) => calc_total_block_count_exc_burst_gaps(version, None, data_par_burst, x),
            None => {
                let file_size = file_utils::get_file_size(&param.in_file)?;
                file_size / block_size as u64
            }
        }
    };

    let stats = Arc::new(Mutex::new(Stats::new(
        &ref_block,
        total_block_count,
        json_printer,
    )));

    let mut reader = FileReader::new(
        &param.in_file,
        FileReaderParam {
            write: !param.dry_run,
            buffered: true,
        },
    )?;

    let mut block = Block::dummy();

    let reporter = Arc::new(ProgressReporter::new(
        &stats,
        "SBX metadata block updating progress",
        "blocks",
        param.pr_verbosity_level,
        param.json_printer.json_enabled(),
    ));

    reporter.start();

    for &p in sbx_block::calc_meta_block_all_write_pos_s(version, data_par_burst).iter() {
        break_if_atomic_bool!(ctrlc_stop_flag);

        reader.seek(SeekFrom::Start(p))?;
        let read_res = reader.read(sbx_block::slice_buf_mut(version, &mut buffer))?;

        break_if_eof_seen!(read_res);

        let block_okay =
            match block.sync_from_buffer(&buffer, Some(&pred)) {
                Ok(()) => true,
                Err(_) => false,
            } &&
            block.is_meta();

        if block_okay {
            update_metas(&block, param.metas_to_add);

            stats.lock().meta_blocks_updated += 1;
        } else {
            stats.lock().meta_blocks_failed += 1;
        }
    }
}
