pub enum Metadata {
    FNM(Box<[u8]>),
    SNM(Box<[u8]>),
    FSZ(u64),
    FDT(u64),
    SDT(u64),
    HSH()
}
