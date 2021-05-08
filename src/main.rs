use std::fs::File;
use std::io;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::thread;
use std::time::Duration;
use std::time::SystemTime;

use clap::{AppSettings, Clap};
use timeout_readwrite::TimeoutReader;

/// Tool to use as pipe to limit what is read from stdin (in time or size).
///
/// Example: `pipe-cutter --tail nginx.log --seconds 10 > 10-secs.log`
///
/// Example: `tail -f nginx.log gnunicorn.log | pipe-cutter --bytes 300`
///
#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    /// Stop reading after that many seconds
    #[clap(long)]
    seconds: Option<u64>,
    /// Stop after reading that many bytes.  If reading from stdin, try to honor
    /// newlines.
    #[clap(long)]
    bytes: Option<usize>,
    /// Read changes in this file ("tail -f" style), as opposed to using stdin
    #[clap(long)]
    tail: Option<String>,
}

fn get_file_reader(path: &str, timeout: Duration) -> Box<dyn Read> {
    let mut file = match File::open(&path) {
        Ok(f) => f,
        Err(error) => {
            eprintln!("Could not open file {}: {}", path, error);
            std::process::exit(1);
        }
    };
    let metadata = match file.metadata() {
        Ok(m) => m,
        Err(error) => {
            eprintln!("Could not read file metadata: {}", error);
            std::process::exit(1);
        }
    };
    if let Err(error) = file.seek(SeekFrom::Start(metadata.len())) {
        eprintln!("Could not jump to end of file: {}", error);
        std::process::exit(1);
    };
    Box::new(TimeoutReader::new(file, timeout))
}

fn get_stdin_reader(timeout: Duration) -> Box<dyn Read> {
    Box::new(TimeoutReader::new(io::stdin(), timeout))
}

fn main() {
    let opts: Opts = Opts::parse();
    if opts.bytes.is_none() && opts.seconds.is_none() {
        eprintln!("Error: you need to specify size and/or time.");
        std::process::exit(2);
    }

    let is_tail = opts.tail.is_some();
    let timeout = Duration::from_millis(250);
    let mut buffer = [0; 4096];
    let mut remaining_bytes = opts.bytes;
    let target = match opts.seconds {
        Some(s) => Some(SystemTime::now() + Duration::new(s, 0)),
        None => None,
    };

    let mut reader = if let Some(path) = opts.tail {
        get_file_reader(&path, timeout)
    } else {
        get_stdin_reader(timeout)
    };

    let done = || {
        if let Some(time) = target {
            SystemTime::now() > time
        } else {
            false
        }
    };

    loop {
        match reader.read(&mut buffer) {
            Ok(0) => {
                // stdin is a buffered reader; a read of 0 implies EOF for our purposes.
                if !is_tail || done() {
                    break;
                }
                thread::sleep(timeout);
            }
            Ok(read_bytes) => {
                let mut to_print = read_bytes;
                if let Some(bytes) = remaining_bytes {
                    // When reading from stdin, we favor printing whole lines
                    if !is_tail && to_print >= bytes {
                        to_print = bytes;
                    }
                }
                if let Err(error) = io::stdout().write_all(&buffer[..to_print]) {
                    eprintln!("Write error: {}", error);
                    std::process::exit(1);
                }
                if let Some(bytes) = remaining_bytes {
                    if read_bytes >= bytes || done() {
                        break;
                    }
                    remaining_bytes = Some(bytes - read_bytes);
                }
            }
            Err(error) if error.kind() == io::ErrorKind::TimedOut => {
                if done() {
                    break;
                }
            }
            Err(error) => {
                eprintln!("Read error: {}", error);
                std::process::exit(1);
            }
        }
    }
}
