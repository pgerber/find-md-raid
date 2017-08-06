use std::{env, fs, io, process};

/// Find Linux md raid magic number 0xa92b4efc.
///
/// The magic number is stored in little endian representation.
fn search<S>(stream: &mut S) -> io::Result<()>
where
    S: io::Read + io::Seek,
{
    let mut buf = [0; 1024 * 1024];
    let mut last_four_bytes = 0_u32;
    let mut offset = 0_usize;

    loop {
        let size = stream.read(&mut buf)?;
        if size == 0 {
            break;
        }
        for byte in &buf[..size] {
            last_four_bytes = last_four_bytes >> 8 | (*byte as u32) << 24;
            if last_four_bytes == 0xa92b4efc {
                println!("hit at byte {}", offset - 3);
            }
            offset += 1;
        }
    }
    Ok(())
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
