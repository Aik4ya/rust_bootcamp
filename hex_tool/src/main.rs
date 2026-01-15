use clap::Parser;
use std::fs::File;
use std::io::{Read, Write, Seek, SeekFrom};

#[derive(Parser)]
#[command(name = "hextool")]
#[command(about = "lit et ecrit des fichiers binaires en hex")]
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
    if hex.len() % 2 != 0 {
        return Err("la string hex doit avoir un nombre pair de caracteres".to_string());
    }
    
    let mut bytes = Vec::new();
    for i in (0..hex.len()).step_by(2) {
        let byte_str = &hex[i..i+2];
        match u8::from_str_radix(byte_str, 16) {
            Ok(byte) => bytes.push(byte),
            Err(_) => return Err(format!("hex pas bon: {}", byte_str)),
        }
    }
    Ok(bytes)
}

fn read_mode(file_path: &str, offset: usize, size: usize) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open(file_path)?;
    file.seek(SeekFrom::Start(offset as u64))?;

    let mut buffer = vec![0; size];
    let bytes_read = file.read(&mut buffer)?;
    buffer.truncate(bytes_read);

    println!("lecture {} bytes a partir de 0x{:08x}", bytes_read, offset);
    println!();

    for (i, chunk) in buffer.chunks(16).enumerate() {
        let pos = offset + i * 16;
        print!("{:08x}: ", pos);

        for byte in chunk {
            print!("{:02x} ", byte);
        }

        for _ in chunk.len()..16 {
            print!("   ");
        }

        print!(" |");
        for byte in chunk {
            let ch = if *byte >= 32 && *byte < 127 {
                *byte as char
            } else {
                '.'
            };
            print!("{}", ch);
        }
        println!("|");
    }

    Ok(())
}

fn write_mode(file_path: &str, hex_string: &str, offset: usize) -> Result<(), Box<dyn std::error::Error>> {
    let bytes = hex_string_to_bytes(hex_string)?;
    
    let mut buffer = if std::path::Path::new(file_path).exists() {
        let mut file = File::open(file_path)?;
        let file_size = file.metadata()?.len() as usize;

        if offset > file_size {
            return Err("offset trop grand".into());
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

    println!("ecriture de {} bytes a l'offset 0x{:08x}", bytes.len(), offset);
    print!("hex:   ");
    for byte in &bytes {
        print!("{:02x} ", byte);
    }
    println!();

    print!("ascii: ");
    for byte in &bytes {
        let ch = if *byte >= 32 && *byte < 127 {
            *byte as char
        } else {
            '?'
        };
        print!("{}", ch);
    }
    println!();
    println!("ok c'est ecrit");

    Ok(())
}

fn main() {
    let args = Args::parse();

    let offset = parse_offset(&args.offset);

    if args.read {
        if let Err(e) = read_mode(&args.file, offset, args.size) {
            eprintln!("oups erreur en lecture: {}", e);
        }
    } else if let Some(hex_data) = args.write {
        if let Err(e) = write_mode(&args.file, &hex_data, offset) {
            eprintln!("oups erreur en ecriture: {}", e);
        }
    } else {
        eprintln!("faut utiliser --read ou --write");
    }
}
