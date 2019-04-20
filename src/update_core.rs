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
        json_printer: Arc<JSONPrinter>,
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
