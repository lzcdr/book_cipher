use base64::{engine::general_purpose::STANDARD, Engine as _};
use clap::Parser;
use reqwest::blocking::Client;
use std::io::{self, IsTerminal, Read, Write};
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(short, long)]
    url: String,

    #[arg(short, long)]
    key: usize,

    #[arg(short = 'r', long)]
    reverse: bool,
}

fn fetch_key(url: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let client = Client::builder().timeout(Duration::from_secs(60)).build()?;
    let resp = client.get(url).send()?.error_for_status()?;
    Ok(resp.bytes()?.to_vec())
}

fn read_stdin_bytes() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut buf = Vec::new();
    io::stdin().read_to_end(&mut buf)?;
    Ok(buf)
}

fn keystream(key: &[u8], offset: usize, len: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(len);
    let klen = key.len();
    for i in 0..len {
        out.push(key[(offset + i) % klen]);
    }
    out
}

fn xor_like(msg: &[u8], ks: &[u8]) -> Vec<u8> {
    msg.iter()
        .zip(ks.iter())
        .map(|(m, k)| m.wrapping_add(*k))
        .collect()
}

fn unxor_like(ct: &[u8], ks: &[u8]) -> Vec<u8> {
    ct.iter()
        .zip(ks.iter())
        .map(|(c, k)| c.wrapping_sub(*k))
        .collect()
}

/// Write raw bytes to stdout, bypassing Rust's UTF-8 validation
/// that `std::io::Stdout` performs on Windows console handles.
#[cfg(windows)]
fn write_stdout_raw(bytes: &[u8]) -> io::Result<()> {
    use std::fs::File;
    use std::os::windows::io::FromRawHandle;

    // (DWORD)-11 == STD_OUTPUT_HANDLE
    const STD_OUTPUT_HANDLE: u32 = 0xFFFF_FFF5;

    extern "system" {
        fn GetStdHandle(nstdhandle: u32) -> *mut core::ffi::c_void;
    }

    let handle = unsafe { GetStdHandle(STD_OUTPUT_HANDLE) };
    if handle.is_null() || handle == (-1isize as *mut core::ffi::c_void) {
        return Err(io::Error::last_os_error());
    }
    let mut f = unsafe { File::from_raw_handle(handle) };
    let r = f.write_all(bytes);
    std::mem::forget(f); // do not close the console handle
    r
}

#[cfg(not(windows))]
fn write_stdout_raw(bytes: &[u8]) -> io::Result<()> {
    io::stdout().write_all(bytes)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let key_bytes = fetch_key(&args.url)?;
    if key_bytes.is_empty() {
        return Err("key text is empty".into());
    }

    let offset = args.key % key_bytes.len();
    let input = read_stdin_bytes()?;

    let stdin_is_tty = io::stdin().is_terminal();
    let stdout_is_tty = io::stdout().is_terminal();

    // Ctrl+D / Ctrl+Z does not echo a newline; add one for interactive mode.
    if stdin_is_tty && stdout_is_tty {
        write_stdout_raw(b"\n")?;
    }

    if args.reverse {
        let s = std::str::from_utf8(&input)?;
        let cleaned: String = s.chars().filter(|c| !c.is_whitespace()).collect();
        let ct = STANDARD.decode(cleaned.as_bytes())?;

        let ks = keystream(&key_bytes, offset, ct.len());
        let pt = unxor_like(&ct, &ks);

        write_stdout_raw(&pt)?;
        if stdout_is_tty {
            write_stdout_raw(b"\n")?;
        }
    } else {
        let ks = keystream(&key_bytes, offset, input.len());
        let ct = xor_like(&input, &ks);
        let encoded = STANDARD.encode(&ct);

        write_stdout_raw(encoded.as_bytes())?;
        write_stdout_raw(b"\n")?;
    }

    Ok(())
}
