use crate::misc_utils::RangeEnd;

pub enum ReadPattern {
    Sequential(Option<(usize, usize, usize)>),
    BurstErrorResistant(usize, usize, usize),
}

impl ReadPattern {
    pub fn new(
        from_pos: Option<u64>,
        to_pos: Option<RangeEnd<u64>>,
        data_par_burst: Option<(usize, usize, usize)>,
    ) -> Self {
        match data_par_burst {
            Some((data, parity, burst)) => match (from_pos, to_pos) {
                (None, None) => ReadPattern::BurstErrorResistant(data, parity, burst),
                _ => ReadPattern::Sequential(Some((data, parity, burst))),
            },
            None => ReadPattern::Sequential(None),
        }
    }
}
