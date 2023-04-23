/// Contains structures and functions to parse and manipulate packets related to RingEdge 2 Maimai Cabinet.
///
/// Note that all Packet structures contains data arrays **WITHOUT SYNC and SUM byte**,
/// because they are only need when writing/reading from devices.
///
pub mod rs232;
