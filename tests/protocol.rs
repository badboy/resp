// Tests adopted from
// https://github.com/antirez/redis/blob/unstable/tests/unit/protocol.tcl

extern crate nom;
extern crate resp;

use resp::array;

#[test]
fn handle_empty_query() {
    assert!(array(b"\r\n").is_err());
}

#[test]
fn negative_multibulk_length() {
    assert!(array(b"*-10\r\n").is_err());
}

#[test]
fn out_of_range_multibulk_length() {
    assert!(array(b"*20000000\r\n").is_incomplete());
    // "*invalid multibulk length*"
}

#[test]
fn wrong_multibulk_payload_header() {
    assert!(array(b"*3\r\n$3\r\nSET\r\n$1\r\nx\r\nfooz\r\n").is_err());
    // "*expected '$', got 'f'*"
}

#[test]
fn negative_multibulk_payload_length() {
    assert!(array(b"*3\r\n$3\r\nSET\r\n$1\r\nx\r\n$-10\r\n").is_err());
    // "*invalid bulk length*"
}

#[test]
fn out_of_range_multibulk_payload_length() {
    assert!(array(b"*3\r\n$3\r\nSET\r\n$1\r\nx\r\n$2000000000\r\n").is_incomplete());
    // "*invalid bulk length*"
}

#[test]
fn non_number_multibulk_payload_length() {
    assert!(array(b"*3\r\n$3\r\nSET\r\n$1\r\nx\r\n$blabla\r\n").is_err());
    // "*invalid bulk length*"
}

#[test]
fn multibulk_request_not_followed_by_bulk_arguments() {
    assert!(array(b"*1\r\nfoo\r\n").is_err());
    // "*expected '$', got 'f'*"
}
