use crate::file_utils;
use crate::misc_utils;
use crate::time_utils;
use std::fmt;
use std::fs;
use std::io::SeekFrom;
use std::sync::{Arc, Mutex};

use crate::misc_utils::RequiredLenAndSeekTo;

use crate::json_printer::{BracketType, JSONPrinter};

use crate::progress_report::*;

use crate::cli_utils::setup_ctrlc_handler;
use std::time::UNIX_EPOCH;

use crate::file_reader::{FileReader, FileReaderParam};
use crate::file_writer::{FileWriter, FileWriterParam};
use crate::reader::{Reader, ReaderType};

use crate::multihash;

use crate::general_error::Error;
use crate::sbx_specs::{Version, SBX_LAST_SEQ_NUM};

use crate::sbx_block::{
    calc_data_block_write_pos, make_too_much_meta_err_string, Block, BlockType, Metadata,
};

use crate::sbx_block;
use crate::sbx_specs::{
    ver_forces_meta_enabled, ver_to_block_size, ver_to_data_size, ver_to_max_data_file_size,
    ver_to_usize, ver_uses_rs, SBX_FILE_UID_LEN, SBX_LARGEST_BLOCK_SIZE,
};

use crate::misc_utils::{PositionOrLength, RangeEnd};

use rayon::prelude::*;

use reed_solomon_erasure::ReedSolomon;

use smallvec::SmallVec;

const DEFAULT_SINGLE_LOT_SIZE: usize = 10;

#[derive(Clone, Debug)]
pub struct Stats {
    uid: [u8; SBX_FILE_UID_LEN],
    version: Version,
    chunk_size: u64,
    hash_bytes: Option<multihash::HashBytes>,
    pub meta_blocks_written: u64,
    pub data_blocks_written: u64,
    pub parity_blocks_written: u64,
    pub data_padding_bytes: usize,
    pub in_file_size: u64,
    pub out_file_size: u64,
    total_data_blocks: Option<u64>,
    start_time: f64,
    end_time: f64,
    json_printer: Arc<JSONPrinter>,
}

impl fmt::Display for Stats {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let rs_enabled = ver_uses_rs(self.version);
        let block_size = ver_to_block_size(self.version);
        let data_size = ver_to_data_size(self.version);
        let meta_blocks_written = self.meta_blocks_written;
        let data_blocks_written = self.data_blocks_written;
        let parity_blocks_written = self.parity_blocks_written;
        let blocks_written = meta_blocks_written + data_blocks_written + parity_blocks_written;
        let data_bytes_encoded = self.data_bytes_encoded();
        let in_file_size = self.in_file_size;
        let out_file_size = self.out_file_size;
        let time_elapsed = (self.end_time - self.start_time) as i64;
        let (hour, minute, second) = time_utils::seconds_to_hms(time_elapsed);

        let json_printer = &self.json_printer;

        json_printer.write_open_bracket(f, Some("stats"), BracketType::Curly)?;

        if rs_enabled {
            write_maybe_json!(
                f,
                json_printer,
                "File UID                               : {}",
                misc_utils::bytes_to_upper_hex_string(&self.uid)
                    => force_quotes
            )?;
            write_maybe_json!(
                f,
                json_printer,
                "SBX version                            : {}",
                ver_to_usize(self.version)
            )?;
            write_maybe_json!(
                f,
                json_printer,
                "Block size used in encoding            : {}",
                block_size
            )?;
            write_maybe_json!(
                f,
                json_printer,
                "Data  size used in encoding            : {}",
                data_size
            )?;
            write_maybe_json!(
                f,
                json_printer,
                "Number of blocks written               : {}",
                blocks_written
            )?;
            write_maybe_json!(
                f,
                json_printer,
                "Number of blocks written (metadata)    : {}",
                meta_blocks_written
            )?;
            write_maybe_json!(
                f,
                json_printer,
                "Number of blocks written (data)        : {}",
                data_blocks_written
            )?;
            write_maybe_json!(
                f,
                json_printer,
                "Number of blocks written (parity)      : {}",
                parity_blocks_written
            )?;
            write_maybe_json!(
                f,
                json_printer,
                "Amount of data encoded (bytes)         : {}",
                data_bytes_encoded
            )?;
            write_maybe_json!(
                f,
                json_printer,
                "File size                              : {}",
                in_file_size
            )?;
            write_maybe_json!(
                f,
                json_printer,
                "SBX container size                     : {}",
                out_file_size
            )?;
            write_maybe_json!(
                f,
                json_printer,
                "Hash                                   : {}",
                match self.hash_bytes {
                    None => null_if_json_else_NA!(json_printer).to_string(),
                    Some(ref h) => format!(
                        "{} - {}",
                        multihash::hash_type_to_string(h.0),
                        misc_utils::bytes_to_lower_hex_string(&h.1)
                    ),
                }
            )?;
            write_maybe_json!(
                f,
                json_printer,
                "Time elapsed                           : {:02}:{:02}:{:02}",
                hour,
                minute,
                second
            )?;
        } else {
            write_maybe_json!(
                f,
                json_printer,
                "File UID                            : {}",
                misc_utils::bytes_to_upper_hex_string(&self.uid)
                    => force_quotes
            )?;
            write_maybe_json!(
                f,
                json_printer,
                "SBX version                         : {}",
                ver_to_usize(self.version)
            )?;
            write_maybe_json!(
                f,
                json_printer,
                "Block size used in encoding         : {}",
                block_size
            )?;
            write_maybe_json!(
                f,
                json_printer,
                "Data  size used in encoding         : {}",
                data_size
            )?;
            write_maybe_json!(
                f,
                json_printer,
                "Number of blocks written            : {}",
                blocks_written
            )?;
            write_maybe_json!(
                f,
                json_printer,
                "Number of blocks written (metadata) : {}",
                meta_blocks_written
            )?;
            write_maybe_json!(
                f,
                json_printer,
                "Number of blocks written (data)     : {}",
                data_blocks_written
            )?;
            write_maybe_json!(
                f,
                json_printer,
                "Amount of data encoded (bytes)      : {}",
                data_bytes_encoded
            )?;
            write_maybe_json!(
                f,
                json_printer,
                "File size                           : {}",
                in_file_size
            )?;
            write_maybe_json!(
                f,
                json_printer,
                "SBX container size                  : {}",
                out_file_size
            )?;
            write_maybe_json!(
                f,
                json_printer,
                "Hash                                : {}",
                match self.hash_bytes {
                    None => null_if_json_else_NA!(json_printer).to_string(),
                    Some(ref h) => format!(
                        "{} - {}",
                        multihash::hash_type_to_string(h.0),
                        misc_utils::bytes_to_lower_hex_string(&h.1)
                    ),
                }
            )?;
            write_maybe_json!(
                f,
                json_printer,
                "Time elapsed                        : {:02}:{:02}:{:02}",
                hour,
                minute,
                second
            )?;
        }

        json_printer.write_close_bracket(f)?;

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Param {
    version: Version,
    uid: [u8; SBX_FILE_UID_LEN],
    data_par_burst: Option<(usize, usize, usize)>,
    rs_enabled: bool,
    meta_enabled: bool,
    json_printer: Arc<JSONPrinter>,
    hash_type: multihash::HashType,
    from_pos: Option<u64>,
    to_pos: Option<RangeEnd<u64>>,
    in_file: Option<String>,
    out_file: String,
    pr_verbosity_level: PRVerbosityLevel,
}

impl Param {
    pub fn new(
        version: Version,
        uid: &[u8; SBX_FILE_UID_LEN],
        data_par_burst: Option<(usize, usize, usize)>,
        meta_enabled: bool,
        json_printer: &Arc<JSONPrinter>,
        hash_type: multihash::HashType,
        from_pos: Option<u64>,
        to_pos: Option<RangeEnd<u64>>,
        in_file: Option<&str>,
        out_file: &str,
        pr_verbosity_level: PRVerbosityLevel,
    ) -> Param {
        Param {
            version,
            uid: uid.clone(),
            data_par_burst,
            rs_enabled: ver_uses_rs(version),
            meta_enabled: ver_forces_meta_enabled(version) || meta_enabled,
            json_printer: Arc::clone(json_printer),
            hash_type,
            from_pos,
            to_pos,
            in_file: match in_file {
                None => None,
                Some(f) => Some(String::from(f)),
            },
            out_file: String::from(out_file),
            pr_verbosity_level,
        }
    }
}

impl Stats {
    pub fn new(param: &Param, required_len: Option<u64>) -> Stats {
        use crate::file_utils::from_orig_file_size::calc_data_chunk_count;
        Stats {
            uid: param.uid,
            version: param.version,
            chunk_size: ver_to_data_size(param.version) as u64,
            hash_bytes: None,
            meta_blocks_written: 0,
            data_blocks_written: 0,
            parity_blocks_written: 0,
            data_padding_bytes: 0,
            total_data_blocks: match required_len {
                Some(len) => Some(calc_data_chunk_count(param.version, len)),
                None => None,
            },
            in_file_size: 0,
            out_file_size: 0,
            start_time: 0.,
            end_time: 0.,
            json_printer: Arc::clone(&param.json_printer),
        }
    }

    pub fn data_bytes_encoded(&self) -> u64 {
        let data_size = ver_to_data_size(self.version);

        self.data_blocks_written as u64 * data_size as u64 - self.data_padding_bytes as u64
    }
}

impl ProgressReport for Stats {
    fn start_time_mut(&mut self) -> &mut f64 {
        &mut self.start_time
    }

    fn end_time_mut(&mut self) -> &mut f64 {
        &mut self.end_time
    }

    fn units_so_far(&self) -> u64 {
        self.data_blocks_written * self.chunk_size
    }

    fn total_units(&self) -> Option<u64> {
        match self.total_data_blocks {
            None => None,
            Some(x) => Some(x * self.chunk_size),
        }
    }
}

struct Lot<'a> {
    version: Version,
    block: Block,
    data_par_burst: Option<(usize, usize, usize)>,
    meta_enabled: bool,
    block_size: usize,
    lot_size: usize,
    data_block_count: usize,
    padding_block_count: usize,
    parity_block_count: usize,
    directly_writable_slots: usize,
    data: Vec<u8>,
    write_pos_s: Vec<u64>,
    rs_codec: Option<&'a ReedSolomon>,
}

struct DataBlockBuffer<'a> {
    lots: Vec<Lot<'a>>,
    lot_size: usize,
    lots_used: usize,
    start_seq_num: Option<u32>,
}

enum GetSlotResult<'a> {
    None,
    Some(&'a mut [u8]),
    LastSlot(&'a mut [u8]),
}

impl<'a> Lot<'a> {
    pub fn new(
        version: Version,
        uid: &[u8; SBX_FILE_UID_LEN],
        data_par_burst: Option<(usize, usize, usize)>,
        meta_enabled: bool,
        lot_size: usize,
        rs_codec: &'a Option<ReedSolomon>,
    ) -> Self {
        let block_size = ver_to_block_size(version);

        let rs_codec = match rs_codec {
            None => None,
            Some(ref c) => Some(c),
        };

        let directly_writable_slots = match data_par_burst {
            None => lot_size,
            Some((data, _, _)) => data,
        };

        Lot {
            version,
            block: Block::new(version, uid, BlockType::Data),
            data_par_burst,
            meta_enabled,
            block_size,
            lot_size,
            data_block_count: 0,
            padding_block_count: 0,
            parity_block_count: 0,
            directly_writable_slots,
            data: vec![0; block_size * lot_size],
            write_pos_s: vec![0; lot_size],
            rs_codec,
        }
    }

    pub fn get_slot(&mut self) -> GetSlotResult {
        if self.slots_used() < self.directly_writable_slots {
            let start = self.slots_used() * self.block_size;
            let end_exc = start + self.block_size;
            self.data_block_count += 1;
            if self.is_full() {
                GetSlotResult::LastSlot(&mut self.data[start..end_exc])
            } else {
                GetSlotResult::Some(&mut self.data[start..end_exc])
            }
        } else {
            GetSlotResult::None
        }
    }

    pub fn cancel_last_slot(&mut self) {
        assert!(self.data_block_count > 0);

        self.data_block_count -= 1;
    }

    pub fn active(&self) -> bool {
        self.slots_used() > 0
    }

    fn fill_in_padding(&mut self) {
        if let Some(_) = self.data_par_burst {
            for i in self.data_block_count..self.directly_writable_slots {
                let start = i * self.block_size;
                let end_exc = start + self.block_size;
                let slot = &mut self.data[start..end_exc];

                sbx_block::write_padding(self.version, 0, slot);

                self.padding_block_count += 1;
            }
        }
    }

    fn rs_encode(&mut self) {
        if let Some(rs_codec) = self.rs_codec {
            let mut refs: SmallVec<[&mut [u8]; 32]> = SmallVec::with_capacity(self.lot_size);

            // collect references to data segments
            for slot in self.data.chunks_mut(self.block_size) {
                refs.push(slot);
            }

            rs_codec.encode(&mut refs).unwrap();

            self.parity_block_count = rs_codec.parity_shard_count();
        }
    }

    fn slots_used(&self) -> usize {
        self.data_block_count + self.padding_block_count + self.parity_block_count
    }

    pub fn is_full(&self) -> bool {
        self.data_block_count == self.directly_writable_slots
    }

    pub fn encode(&mut self, lot_start_seq_num: u32) {
        if self.active() {
            self.fill_in_padding();

            self.rs_encode();

            let slots_used = self.slots_used();

            for (slot_index, slot) in self.data.chunks_mut(self.block_size).enumerate() {
                if slot_index < slots_used {
                    let tentative_seq_num = lot_start_seq_num as u64 + slot_index as u64;

                    assert!(tentative_seq_num <= SBX_LAST_SEQ_NUM as u64);

                    self.block
                        .set_seq_num(lot_start_seq_num + slot_index as u32);

                    self.block.sync_to_buffer(None, slot).unwrap();

                    let write_pos = calc_data_block_write_pos(
                        self.version,
                        self.block.get_seq_num(),
                        Some(self.meta_enabled),
                        self.data_par_burst,
                    );

                    self.write_pos_s[slot_index] = write_pos;
                } else {
                    break;
                }
            }
        }
    }

    fn reset(&mut self) {
        self.data_block_count = 0;
        self.padding_block_count = 0;
        self.parity_block_count = 0;
    }

    pub fn data_padding_parity_block_count(&self) -> (usize, usize, usize) {
        (
            self.data_block_count,
            self.padding_block_count,
            self.parity_block_count,
        )
    }

    pub fn write(&mut self, writer: &mut FileWriter) -> Result<(), Error> {
        if self.active() {
            let slots_used = self.slots_used();

            for (slot_index, slot) in self.data.chunks_mut(self.block_size).enumerate() {
                if slot_index < slots_used {
                    let write_pos = self.write_pos_s[slot_index];

                    writer.seek(SeekFrom::Start(write_pos))?;

                    writer.write(slot)?;
                } else {
                    break;
                }
            }

            self.reset();
        }

        Ok(())
    }
}

impl<'a> DataBlockBuffer<'a> {
    pub fn new(
        version: Version,
        uid: &[u8; SBX_FILE_UID_LEN],
        data_par_burst: Option<(usize, usize, usize)>,
        meta_enabled: bool,
        lot_size: usize,
        lot_count: usize,
        rs_codec: &'a Option<ReedSolomon>,
    ) -> Self {
        assert!(lot_count > 0);

        let mut lots = Vec::with_capacity(lot_count);

        for _ in 0..lot_count {
            lots.push(Lot::new(
                version,
                uid,
                data_par_burst,
                meta_enabled,
                lot_size,
                rs_codec,
            ))
        }

        DataBlockBuffer {
            lots,
            lot_size,
            lots_used: 0,
            start_seq_num: Some(1),
        }
    }

    pub fn lot_count(&self) -> usize {
        self.lots.len()
    }

    pub fn is_full(&self) -> bool {
        self.lots_used == self.lot_count()
    }

    pub fn active(&self) -> bool {
        self.lots_used > 0 || self.lots[self.lots_used].slots_used() > 0
    }

    pub fn get_slot(&mut self) -> Option<&mut [u8]> {
        let lot_count = self.lots.len();

        if self.lots_used == lot_count {
            None
        } else {
            match self.lots[self.lots_used].get_slot() {
                GetSlotResult::LastSlot(slot) => {
                    self.lots_used += 1;
                    Some(slot)
                }
                GetSlotResult::Some(slot) => Some(slot),
                GetSlotResult::None => {
                    self.lots_used += 1;
                    None
                }
            }
        }
    }

    pub fn cancel_last_slot(&mut self) {
        assert!(self.active());

        let shift_back_one_slot = self.is_full() || self.lots[self.lots_used].slots_used() == 0;

        if shift_back_one_slot {
            self.lots_used -= 1;
        }

        self.lots[self.lots_used].cancel_last_slot();
    }

    pub fn encode(&mut self) -> Result<(), Error> {
        let start_seq_num = self.start_seq_num.unwrap();
        let lot_size = self.lot_size;

        self.lots
            .par_iter_mut()
            .enumerate()
            .for_each(|(lot_index, lot)| {
                let lot_start_seq_num = start_seq_num + (lot_index * lot_size) as u32;

                lot.encode(lot_start_seq_num);
            });

        if self.is_full() {
            self.start_seq_num = Some(start_seq_num + (self.lots.len() * lot_size) as u32);
        } else {
            self.start_seq_num = None;
        }

        Ok(())
    }

    fn reset(&mut self) {
        self.lots_used = 0;

        for lot in self.lots.iter_mut() {
            lot.reset();
        }
    }

    pub fn data_padding_parity_block_count(&self) -> (usize, usize, usize) {
        let mut data_blocks = 0;
        let mut padding_blocks = 0;
        let mut parity_blocks = 0;

        for lot in self.lots.iter() {
            let (data, padding, parity) = lot.data_padding_parity_block_count();

            data_blocks += data;
            padding_blocks += padding;
            parity_blocks += parity;
        }

        (data_blocks, padding_blocks, parity_blocks)
    }

    pub fn write(&mut self, writer: &mut FileWriter) -> Result<(), Error> {
        for lot in self.lots.iter_mut() {
            lot.write(writer)?;
        }

        self.reset();

        Ok(())
    }
}

fn pack_metadata(
    block: &mut Block,
    param: &Param,
    stats: &Stats,
    file_metadata: &Option<fs::Metadata>,
    file_size: Option<u64>,
    hash: Option<multihash::HashBytes>,
) {
    block.set_seq_num(0);

    let metas = block.metas_mut().unwrap();

    {
        // add file name
        match param.in_file {
            None => {}
            Some(ref f) => {
                let file_name = file_utils::get_file_name_part_of_path(f);
                metas.push(Metadata::FNM(file_name));
            }
        }
    }
    {
        // add SBX file name
        let file_name = file_utils::get_file_name_part_of_path(&param.out_file);
        metas.push(Metadata::SNM(file_name));
    }
    {
        // add file size
        match file_size {
            Some(f) => metas.push(Metadata::FSZ(f)),
            None => {}
        }
    }
    {
        // add file last modifcation time
        match file_metadata {
            &Some(ref m) => match m.modified() {
                Ok(t) => match t.duration_since(UNIX_EPOCH) {
                    Ok(t) => metas.push(Metadata::FDT(t.as_secs() as i64)),
                    Err(_) => {}
                },
                Err(_) => {}
            },
            &None => {}
        }
    }
    {
        // add SBX encoding time
        metas.push(Metadata::SDT(stats.start_time as i64));
    }
    {
        // add hash
        let hsh = match hash {
            Some(hsh) => hsh,
            None => {
                let ctx = multihash::hash::Ctx::new(param.hash_type).unwrap();
                ctx.finish_into_hash_bytes()
            }
        };
        metas.push(Metadata::HSH(hsh));
    }
    {
        // add RS params
        if param.rs_enabled {
            metas.push(Metadata::RSD(param.data_par_burst.unwrap().0 as u8));
            metas.push(Metadata::RSP(param.data_par_burst.unwrap().1 as u8));
        }
    }
}

fn write_meta_blocks(
    param: &Param,
    stats: &Arc<Mutex<Stats>>,
    file_metadata: &Option<fs::Metadata>,
    file_size: Option<u64>,
    hash: Option<multihash::HashBytes>,
    writer: &mut FileWriter,
    record_stats: bool,
) -> Result<(), Error> {
    let mut block = Block::new(param.version, &param.uid, BlockType::Meta);
    let mut buffer: [u8; SBX_LARGEST_BLOCK_SIZE] = [0; SBX_LARGEST_BLOCK_SIZE];

    // pack metadata into the block
    pack_metadata(
        &mut block,
        param,
        &stats.lock().unwrap(),
        file_metadata,
        file_size,
        hash,
    );

    match block.sync_to_buffer(None, &mut buffer) {
        Ok(()) => {}
        Err(sbx_block::Error::TooMuchMetadata(ref m)) => {
            return Err(Error::with_msg(&make_too_much_meta_err_string(
                block.get_version(),
                m,
            )));
        }
        Err(_) => unreachable!(),
    }

    let write_pos_s =
        sbx_block::calc_meta_block_all_write_pos_s(param.version, param.data_par_burst);

    for &p in write_pos_s.iter() {
        writer.seek(SeekFrom::Start(p))?;

        writer.write(sbx_block::slice_buf(block.get_version(), &buffer))?;

        if record_stats {
            stats.lock().unwrap().meta_blocks_written += 1;
        }
    }

    Ok(())
}

pub fn encode_file(param: &Param) -> Result<Stats, Error> {
    let ctrlc_stop_flag = setup_ctrlc_handler(param.json_printer.json_enabled());

    // setup file reader and writer
    let mut reader = match param.in_file {
        Some(ref f) => Reader::new(ReaderType::File(FileReader::new(
            f,
            FileReaderParam {
                write: false,
                buffered: true,
            },
        )?)),
        None => Reader::new(ReaderType::Stdin(std::io::stdin())),
    };

    let mut writer = FileWriter::new(
        &param.out_file,
        FileWriterParam {
            read: false,
            append: false,
            truncate: true,
            buffered: true,
        },
    )?;

    let metadata = match reader.metadata() {
        Some(m) => Some(m?),
        None => None,
    };

    let file_size = match reader.get_file_size() {
        Some(s) => Some(s?),
        None => None,
    };

    // calulate length to read and position to seek to
    let (required_len, seek_to) = match file_size {
        Some(file_size) => {
            let RequiredLenAndSeekTo {
                required_len,
                seek_to,
            } = misc_utils::calc_required_len_and_seek_to_from_byte_range(
                param.from_pos,
                param.to_pos,
                true,
                0,
                PositionOrLength::Len(file_size),
                None,
            );
            (Some(required_len), Some(seek_to))
        }
        None => (None, None),
    };

    {
        // check if required length exceeds maximum
        match required_len {
            None => {}
            Some(required_len) => {
                let max_in_file_size = ver_to_max_data_file_size(param.version);

                if required_len > max_in_file_size {
                    return Err(Error::with_msg(&format!("Encoding range specified for \"{}\" exceeds the maximum supported file size, size to be encoded : {}, max : {}",
                                                            param.in_file.as_ref().unwrap(),
                                                            required_len,
                                                            max_in_file_size)));
                }
            }
        }
    }

    // setup stats
    let stats = Arc::new(Mutex::new(Stats::new(param, required_len)));

    // setup reporter
    let reporter = ProgressReporter::new(
        &stats,
        "Data encoding progress",
        "bytes",
        param.pr_verbosity_level,
        param.json_printer.json_enabled(),
    );

    // set up hash state
    let mut hash_ctx = multihash::hash::Ctx::new(param.hash_type).unwrap();

    // setup Reed-Solomon things
    let rs_codec = match param.data_par_burst {
        None => None,
        Some((data, parity, _)) => Some(ReedSolomon::new(data, parity).unwrap()),
    };

    let lot_size = match param.data_par_burst {
        None => DEFAULT_SINGLE_LOT_SIZE,
        Some((data, parity, _)) => data + parity,
    };

    // let lot_count = num_cpus::get();
    let lot_count = 1;

    // setup main data buffer
    let mut buffer = DataBlockBuffer::new(
        param.version,
        &param.uid,
        param.data_par_burst,
        param.meta_enabled,
        lot_size,
        lot_count,
        &rs_codec,
    );

    // seek to calculated position
    if let Some(seek_to) = seek_to {
        if let Some(r) = reader.seek(SeekFrom::Start(seek_to)) {
            r?;
        }
    }

    let mut bytes_processed: u64 = 0;

    let mut end_loop = false;

    let mut block_for_seq_num_check = Block::dummy();

    block_for_seq_num_check.set_seq_num(0);

    reporter.start();

    if param.meta_enabled {
        // write dummy metadata block
        write_meta_blocks(
            param,
            &stats,
            &metadata,
            required_len,
            None,
            &mut writer,
            true,
        )?;
    }

    while !end_loop {
        let mut stats = stats.lock().unwrap();

        // fill up data buffer
        while !buffer.is_full() {
            break_if_atomic_bool!(ctrlc_stop_flag);

            if let Some(required_len) = required_len {
                if bytes_processed >= required_len {
                    end_loop = true;
                    break;
                }
            }

            {
                // read data in
                let slot = buffer.get_slot().unwrap();
                let read_res = reader.read(sbx_block::slice_data_buf_mut(param.version, slot))?;

                bytes_processed += read_res.len_read as u64;

                if read_res.len_read == 0 {
                    buffer.cancel_last_slot();
                    end_loop = true;
                    break;
                }

                if let Err(_) = block_for_seq_num_check.add1_seq_num() {
                    return Err(Error::with_msg("Block seq num already at max, addition causes overflow. This might be due to file size being changed during the encoding, or too much data from stdin"));
                }

                hash_ctx
                    .update(&sbx_block::slice_data_buf(param.version, slot)[..read_res.len_read]);

                stats.data_padding_bytes +=
                    sbx_block::write_padding(param.version, read_res.len_read, slot);
            }
        }

        break_if_atomic_bool!(ctrlc_stop_flag);

        buffer.encode()?;

        let (data_blocks, _, parity_blocks) = buffer.data_padding_parity_block_count();

        stats.data_blocks_written += data_blocks as u64;
        stats.parity_blocks_written += parity_blocks as u64;

        buffer.write(&mut writer)?;
    }

    let data_bytes_encoded = match required_len {
        Some(x) => x,
        None => stats.lock().unwrap().data_bytes_encoded(),
    };

    if param.meta_enabled {
        let hash_bytes = hash_ctx.finish_into_hash_bytes();

        // write actual medata blocks
        write_meta_blocks(
            param,
            &stats,
            &metadata,
            Some(data_bytes_encoded),
            Some(hash_bytes.clone()),
            &mut writer,
            false,
        )?;

        // record hash in stats
        stats.lock().unwrap().hash_bytes = Some(hash_bytes.clone());
    }

    reporter.stop();

    stats.lock().unwrap().in_file_size = data_bytes_encoded;
    stats.lock().unwrap().out_file_size = file_utils::from_orig_file_size::calc_container_size(
        param.version,
        Some(param.meta_enabled),
        param.data_par_burst,
        data_bytes_encoded,
    );

    let stats = stats.lock().unwrap().clone();

    Ok(stats)
}
