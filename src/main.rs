extern crate bytes;
extern crate chrono;

use bytes::ByteOrder;
use std::{env, fs, io, process};
use std::io::Read;
use chrono::{DateTime, Local, NaiveDateTime, Utc};

/// Find Linux md raid magic number 0xa92b4efc.
///
/// The magic number is stored in little endian representation.
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
        if buf.starts_with(&[0xfc, 0x4e, 0x2b, 0xa9]) {
            print_hit(offset, &buf);
        }
        offset += buf.len();
    }
}

fn print_hit(offset: usize, block: &[u8; 512]) {
    let version = bytes::LittleEndian::read_u32(&block[4..8]);

    let name = match version {
        1 => {
            let mut n = vec![b'"'];
            n.extend(block[32..64].iter().cloned().take_while(|&c| c != b'\0'));
            n.push(b'"');
            String::from_utf8_lossy(&n).into()
        }
        _ => "unknown".to_string(),
    };

    let timestamp = match version {
        0 => {
            let secs = bytes::LittleEndian::read_u32(&block[24..28]);
            fmt_timestamp(secs as i64, 0)
        }
        1 => {
            let raw = bytes::LittleEndian::read_u64(&block[64..72]);
            let secs = raw & 0xff_ffff_ffff;
            let nsecs = (raw >> 40) * 1000;
            fmt_timestamp(secs as i64, nsecs as u32)
        }
        _ => "unknown".to_string(),
    };

    println!(
        "hit at byte {} (version: {}.x, name: {}, creation time: {})",
        offset,
        version,
        name,
        timestamp
    );
}

fn fmt_timestamp(secs: i64, nsecs: u32) -> String {
    if let Some(ts) = NaiveDateTime::from_timestamp_opt(secs as i64, nsecs as u32) {
        let ts = DateTime::<Utc>::from_utc(ts, Utc);
        let ts = ts.with_timezone(&Local);
        ts.to_rfc3339()
    } else {
        "invalid".to_string()
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
