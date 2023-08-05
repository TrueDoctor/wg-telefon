struct datagram {
    seq: u16,
    datagram_type: u8,
    data: [u8; 1450]
}