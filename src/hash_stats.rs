use crate::progress_report::ProgressReport;

#[derive(Clone, Debug)]
pub struct HashStats {
    pub bytes_processed: u64,
    pub total_bytes: u64,
    start_time: f64,
    end_time: f64,
}

impl HashStats {
    pub fn new(file_size: u64) -> HashStats {
        HashStats {
            bytes_processed: 0,
            total_bytes: file_size,
            start_time: 0.,
            end_time: 0.,
        }
    }
}

impl ProgressReport for HashStats {
    fn start_time_mut(&mut self) -> &mut f64 {
        &mut self.start_time
    }

    fn end_time_mut(&mut self) -> &mut f64 {
        &mut self.end_time
    }

    fn units_so_far(&self) -> u64 {
        self.bytes_processed
    }

    fn total_units(&self) -> Option<u64> {
        Some(self.total_bytes)
    }
}
