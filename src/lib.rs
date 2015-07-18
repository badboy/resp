#[macro_use]
extern crate nom;

use nom::digit;
use std::str::{FromStr,from_utf8_unchecked};

use Value::*;

fn buf_to_usize(s: &[u8]) -> usize {
    FromStr::from_str(unsafe{from_utf8_unchecked(s)}).unwrap()
}
fn buf_to_i64(s: &[u8]) -> i64 {
    FromStr::from_str(unsafe{from_utf8_unchecked(s)}).unwrap()
}

#[derive(Debug,Eq,PartialEq,Clone)]
pub enum Value {
    Integer(i64),
    Status(String),
    Error(String),
    BulkString(Vec<u8>),
    Array(Vec<Value>),
    Nil,
}

named!(digits_as_usize <&[u8], usize>, map!(digit, buf_to_usize));
named!(nil <&[u8], Value>, map!(tag!("$-1\r\n"), |_| Nil));
named!(string <&[u8], Value>,
       chain!(
                  tag!("$")     ~
           len :  digits_as_usize ~
                  tag!("\r\n")      ~
           value: take!(len) ~
                  tag!("\r\n")
           ,
           || { BulkString(value.to_vec()) }
       )
);

/// Parse a length-prefixed binary-safe string
named!(pub bulk_string <&[u8], Value>, alt!(nil | string));

/// Parse a plus-prefixed human-readable string
named!(pub status <&[u8], Value>,
       chain!(
                tag!("+") ~
           val: take_until!("\r\n") ~
                tag!("\r\n")
           ,
           || { Status(String::from_utf8(val.to_owned()).unwrap()) }
           )
       );

/// Parse a plus-prefixed human-readable error string
named!(pub error <&[u8], Value>,
       chain!(
                tag!("-") ~
           val: take_until!("\r\n") ~
                tag!("\r\n")
           ,
           || { Error(String::from_utf8(val.to_owned()).unwrap()) }
           )
       );

/// Parse a signed 64-bit integer
named!(pub integer <&[u8], Value>,
       chain!(
                 tag!(":") ~
           sign: map!(tag!("-"), |_| -1)? ~
           val:  map!(digit, buf_to_i64) ~
                 tag!("\r\n")
           ,
           || { Integer(sign.unwrap_or(1) as i64 *val) }
           )
       );

named!(value <&[u8], Value>, alt!(integer | bulk_string));

named!(null_array <&[u8], Value>,
       map!(tag!("*-1\r\n"), |_| Nil));
named!(filled_array <&[u8], Value >,
       chain!(
                  tag!("*")     ~
            argc: digits_as_usize ~
                  tag!("\r\n")  ~
            argv: count!(value, argc)
            ,
            || {
                assert!(argv.len() == argc);
                Array(argv.to_vec())
            }

       )
);
/// Parse an array of heterogeneous values
///
/// Nil is allowed
named!(pub array <&[u8], Value >,
       alt!(null_array | filled_array));

#[cfg(test)]
mod tests {
    use nom::IResult;
    use super::*;
    use super::Value::*;

    #[test]
    fn parse_array() {
        let data = b"*2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n";

        assert_eq!(
            IResult::Done(&b""[..], Array(vec![
                                          BulkString(b"foo".to_vec()),
                                          BulkString(b"bar".to_vec()),
            ])),
            array(data)
            );
    }

    #[test]
    fn parse_integer() {
        assert_eq!(IResult::Done(&b""[..], Integer(0)), integer(b":0\r\n"));
        assert_eq!(IResult::Done(&b""[..], Integer(-1000)), integer(b":-1000\r\n"));
    }

    #[test]
    fn parse_status() {
        assert_eq!(IResult::Done(&b""[..], Status("OK".to_owned())), status(b"+OK\r\n"));
    }

    #[test]
    fn parse_error() {
        assert_eq!(IResult::Done(&b""[..], Error("Error message".to_owned())), error(b"-Error message\r\n"));
    }

    #[test]
    fn parse_bulk_string() {
        assert_eq!(IResult::Done(&b""[..], BulkString(b"foo".to_vec())), bulk_string(b"$3\r\nfoo\r\n"));
        assert_eq!(IResult::Done(&b""[..], BulkString(b"".to_vec())), bulk_string(b"$0\r\n\r\n"));
        assert_eq!(IResult::Done(&b""[..], Nil), bulk_string(b"$-1\r\n"));
    }

    #[test]
    fn parse_mixed_array() {
        let data = b"*5\r\n:1\r\n:2\r\n:3\r\n:4\r\n$6\r\nfoobar\r\n";
        assert_eq!(
            IResult::Done(&b""[..],
                          Array(vec![
                                Integer(1),
                                Integer(2),
                                Integer(3),
                                Integer(4),
                                BulkString(b"foobar".to_vec()),
                          ])
                         ),
                         array(data)
                  );

        let data = b"*3\r\n$3\r\nfoo\r\n$-1\r\n$3\r\nbar\r\n";
        assert_eq!(
            IResult::Done(&b""[..],
                          Array(vec![
                                BulkString(b"foo".to_vec()),
                                Nil,
                                BulkString(b"bar".to_vec()),
                          ])
                         ),
                         array(data)
                  );
    }
}
