use std::{env, fs, io, process};
use std::io::Read;

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
            println!("hit at byte {}", offset);
        }
        offset += buf.len();
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
