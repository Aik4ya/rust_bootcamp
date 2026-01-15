use clap::Parser;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};

#[derive(Parser)]
#[command(name = "hex_tool")]
#[command(about = "Read and write binary files in hexadecimal")]
struct Args {
    #[arg(short, long)]
    file: String,

    #[arg(short, long)]
    read: bool,

    #[arg(short, long)]
    write: Option<String>,

    #[arg(short, long, default_value = "0")]
    offset: String,

    #[arg(short, long, default_value = "16")]
    size: usize,
}

fn parse_offset(offset_str: &str) -> usize {
    if offset_str.starts_with("0x") || offset_str.starts_with("0X") {
        usize::from_str_radix(&offset_str[2..], 16).unwrap_or(0)
    } else {
        offset_str.parse().unwrap_or(0)
    }
}

fn hex_string_to_bytes(hex: &str) -> Result<Vec<u8>, String> {
    let hex = hex.trim();
    if !hex.len().is_multiple_of(2) {
        return Err("Hex string must have even length".to_string());
    }

    let mut bytes = Vec::new();
    for i in (0..hex.len()).step_by(2) {
        let byte_str = &hex[i..i + 2];
        match u8::from_str_radix(byte_str, 16) {
            Ok(byte) => bytes.push(byte),
            Err(_) => return Err(format!("Invalid hex: {}", byte_str)),
        }
    }
    Ok(bytes)
}

fn read_mode(
    file_path: &str,
    offset: usize,
    size: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open(file_path)?;
    file.seek(SeekFrom::Start(offset as u64))?;

    let mut buffer = vec![0; size];
    let bytes_read = file.read(&mut buffer)?;
    buffer.truncate(bytes_read);

    for (i, chunk) in buffer.chunks(16).enumerate() {
        let pos = offset + i * 16;
        print!("{:08x}:", pos);

        for j in 0..16 {
            print!(" ");
            if j < chunk.len() {
                print!("{:02x}", chunk[j]);
            } else {
                print!("..");
            }
        }

        print!(" |");
        for j in 0..16 {
            if j < chunk.len() {
                let ch = if chunk[j] >= 32 && chunk[j] < 127 {
                    chunk[j] as char
                } else {
                    '.'
                };
                print!("{}", ch);
            } else {
                print!(".");
            }
        }
        println!("|");
    }

    Ok(())
}

fn write_mode(
    file_path: &str,
    hex_string: &str,
    offset: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let bytes = hex_string_to_bytes(hex_string)?;

    let mut buffer = if std::path::Path::new(file_path).exists() {
        let mut file = File::open(file_path)?;
        let file_size = file.metadata()?.len() as usize;

        if offset > file_size {
            return Err("Offset exceeds file size".into());
        }

        let mut buf = vec![0; file_size];
        file.read_exact(&mut buf)?;
        buf
    } else {
        vec![0; offset + bytes.len()]
    };

    for (i, byte) in bytes.iter().enumerate() {
        if offset + i < buffer.len() {
            buffer[offset + i] = *byte;
        } else {
            buffer.push(*byte);
        }
    }

    let mut file = File::create(file_path)?;
    file.write_all(&buffer)?;

    println!("Successfully written");

    Ok(())
}

fn main() {
    let args = Args::parse();

    let offset = parse_offset(&args.offset);

    if args.read {
        if let Err(e) = read_mode(&args.file, offset, args.size) {
            eprintln!("Error reading file: {}", e);
        }
    } else if let Some(hex_data) = args.write {
        if let Err(e) = write_mode(&args.file, &hex_data, offset) {
            eprintln!("Error writing file: {}", e);
        }
    } else {
        eprintln!("Use --read or --write option");
    }
}
