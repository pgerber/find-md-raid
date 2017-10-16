extern crate bytes;
extern crate chrono;

use bytes::{BigEndian, ByteOrder, LittleEndian};
use std::{env, fmt, fs, io, process};
use std::io::Read;
use chrono::{Local, TimeZone};

#[derive(Clone, Copy, PartialEq)]
enum Endian {
    Little,
    Big,
}

impl fmt::Display for Endian {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Endian::Little => write!(f, "little"),
            Endian::Big => write!(f, "big"),
        }
    }
}

/// Find Linux md raid magic number 0xa92b4efc.
fn search<S>(stream: &mut S) -> io::Result<()>
where
    S: io::Read + io::Seek,
{
    let mut buf = [0; 512];
    let mut offset = 0_usize;
    let mut buf_stream = io::BufReader::with_capacity(1048576, stream);

    loop {
        match buf_stream.read_exact(&mut buf) {
            Err(ref e) if e.kind() == io::ErrorKind::UnexpectedEof => break Ok(()),
            e @ Err(_) => break e,
            Ok(()) => (),
        }

        // Version 1.x metadata uses little-endian and so does v0.90 metadata on little-endian
        // platforms
        if buf.starts_with(&[0xfc, 0x4e, 0x2b, 0xa9]) {
            print_hit(offset, &buf, Endian::Little);
        }

        // Version 0.90 metadata uses big-endian representation on big-endian systems.
        if cfg!(any(target_endian = "big", feature = "big_endian")) &&
            buf.starts_with(&[0xa9, 0x2b, 0x4e, 0xfc])
        {
            print_hit(offset, &buf, Endian::Big);
        }
        offset += buf.len();
    }
}

fn print_hit(offset: usize, block: &[u8; 512], endianess: Endian) {
    let version = extract_version(block, endianess);

    if cfg!(any(target_endian = "big", feature = "big_endian")) && endianess == Endian::Big &&
        version.0 != 0
    {
        return; // not valid metadata, only 0.x may be big-endian
    };

    let name = match version {
        (1, _, _) => {
            let mut n = vec![b'"'];
            n.extend(block[32..64].iter().cloned().take_while(|&c| c != b'\0'));
            n.push(b'"');
            String::from_utf8_lossy(&n).into()
        }
        _ => "unknown".to_string(),
    };

    let ctime = match version {
        (0, Some(90), _) => {
            let secs = match endianess {
                Endian::Big => BigEndian::read_u32(&block[24..28]),
                Endian::Little => LittleEndian::read_u32(&block[24..28]),
            };
            fmt_timestamp(secs as i64, 0)
        }
        (1, _, _) => {
            let (secs, nsecs) = extract_64bit_timestamp(&block[64..72]);
            fmt_timestamp(secs, nsecs)
        }
        _ => "unknown".to_string(),
    };

    let utime = match version {
        (0, Some(90), _) => {
            let secs = match endianess {
                Endian::Big => BigEndian::read_u32(&block[24..28]),
                Endian::Little => LittleEndian::read_u32(&block[24..28]),
            };
            fmt_timestamp(secs as i64, 0)
        }
        (1, _, _) => {
            let (secs, nsecs) = extract_64bit_timestamp(&block[192..200]);
            fmt_timestamp(secs, nsecs)
        }
        _ => "unknown".to_string(),
    };

    print!(
        "hit at byte {} (version: {}, name: {}, creation time: {}, update time: {}",
        offset,
        version_string(version),
        name,
        ctime,
        utime,
    );
    if cfg!(any(target_endian = "big", feature = "big_endian")) && version.0 == 0 {
        print!(", endianess: {}", endianess);
    }
    println!(")");
}

fn version_string(version: (u32, Option<u32>, Option<u32>)) -> String {
    match version {
        (major, Some(minor), Some(patch)) => format!("{}.{}.{}", major, minor, patch),
        (major, _, _) => format!("{}.x", major),
    }
}

fn extract_version(data: &[u8; 512], endianess: Endian) -> (u32, Option<u32>, Option<u32>) {
    let major = match endianess {
        Endian::Big => BigEndian::read_u32(&data[4..8]),
        Endian::Little => LittleEndian::read_u32(&data[4..8]),
    };
    let minor = if major == 0 {
        match endianess {
            Endian::Big => Some(BigEndian::read_u32(&data[8..12])),
            Endian::Little => Some(LittleEndian::read_u32(&data[8..12])),
        }
    } else {
        None
    };
    let patch = if major == 0 {
        match endianess {
            Endian::Big => Some(BigEndian::read_u32(&data[12..16])),
            Endian::Little => Some(LittleEndian::read_u32(&data[12..16])),
        }
    } else {
        None
    };
    (major, minor, patch)
}

fn extract_64bit_timestamp(stamp: &[u8]) -> (i64, u32) {
    debug_assert_eq!(stamp.len(), 8);
    let raw = LittleEndian::read_u64(stamp);
    let secs = raw & 0xff_ffff_ffff;
    let nsecs = (raw >> 40) * 1000;
    (secs as i64, nsecs as u32)
}

fn fmt_timestamp(secs: i64, nsecs: u32) -> String {
    match Local.timestamp_opt(secs, nsecs).earliest() {
        Some(ts) => ts.to_rfc3339(),
        None => "invalid".to_string(),
    }
}

fn main() {
    let mut args = env::args_os().skip(1);
    let path = match args.next() {
        Some(path) => path,
        None => {
            println!(
                "error: Too few arguments. Expected exactly one argument, the path to \
                 the device to scan."
            );
            process::exit(1)
        }
    };

    if args.next().is_some() {
        println!(
            "error: Too many arguments. Expected exactly one argument, the path to \
             the device to scan."
        );
        process::exit(1)
    }

    let mut file = match fs::File::open(&path) {
        Ok(file) => file,
        Err(e) => {
            println!("error: failed to open file {:?}: {}", path, e);
            process::exit(1)
        }
    };

    if let Err(e) = search(&mut file) {
        println!("error: failure while scanning {:?}: {}", path, e);
        process::exit(1)
    };
}
