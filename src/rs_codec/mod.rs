macro_rules! assert_not_ready {
    (
        $self:ident
    ) => {{
        assert!(!codec_ready!($self));
    }};
}

macro_rules! assert_ready {
    (
        $self:ident
    ) => {{
        assert!(codec_ready!($self));
    }};
}

mod encoder;
mod encoder_tests;
pub use self::encoder::RSEncoder;

mod repairer;
mod repairer_tests;
pub use self::repairer::RSRepairStats;
pub use self::repairer::RSRepairer;

#[must_use]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RSCodecState {
    Ready,
    NotReady,
}
