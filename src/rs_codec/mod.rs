mod encoder;
pub use self::encoder::RSEncoder;

mod repairer;
pub use self::repairer::RSRepairer;
pub use self::repairer::RSRepairStats;

mod tests;

#[must_use]
pub enum RSCodecState {
    Ready,
    NotReady
}
