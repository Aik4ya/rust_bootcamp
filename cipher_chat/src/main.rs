use clap::{Parser, Subcommand};
use rand::Rng;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::thread;

const P: u64 = 0xD87FA3E291B4C7F3;
const G: u64 = 2;

#[derive(Parser)]
#[command(name = "streamchat")]
#[command(about = "Stream cipher chat with Diffie-Hellman key generation")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Server { port: u16 },
    Client { address: String },
}

fn mod_exp(base: u64, exp: u64, modulus: u64) -> u64 {
    if modulus == 1 {
        return 0;
    }
    let mut result: u128 = 1;
    let mut base = (base as u128) % (modulus as u128);
    let mut exp = exp;
    let modulus = modulus as u128;

    while exp > 0 {
        if exp & 1 == 1 {
            result = (result * base) % modulus;
        }
        exp >>= 1;
        base = (base * base) % modulus;
    }
    result as u64
}

struct KeystreamGenerator {
    state: u64,
}

impl KeystreamGenerator {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_byte(&mut self) -> u8 {
        const A: u64 = 1103515245;
        const C: u64 = 12345;
        const M: u64 = 1 << 31;

        self.state = self.state.wrapping_mul(A).wrapping_add(C) % M;
        (self.state >> 16) as u8
    }
}

fn xor_cipher(data: &[u8], keystream: &mut KeystreamGenerator) -> Vec<u8> {
    data.iter().map(|b| b ^ keystream.next_byte()).collect()
}

fn format_hex_spaced(value: u64) -> String {
    format!(
        "{:04X} {:04X} {:04X} {:04X}",
        (value >> 48) & 0xFFFF,
        (value >> 32) & 0xFFFF,
        (value >> 16) & 0xFFFF,
        value & 0xFFFF
    )
}

fn perform_dh_exchange(stream: &mut TcpStream, is_server: bool) -> std::io::Result<u64> {
    println!("\n[DH] Starting key exchange...");
    println!("[DH] Using hardcoded DH parameters:");
    println!("     p = {} (64-bit prime - public)", format_hex_spaced(P));
    println!("     g = {} (generator - public)", G);

    println!("\n[DH] Generating our keypair...");
    let private_key: u64 = rand::thread_rng().gen();
    println!("     private_key = {:016X} (random 64-bit)", private_key);

    let public_key = mod_exp(G, private_key, P);
    println!(
        "     public_key  = g^private mod p = {:016X}",
        public_key
    );

    let their_public_key: u64;

    if is_server {
        let mut buf = [0u8; 8];
        stream.read_exact(&mut buf)?;
        their_public_key = u64::from_be_bytes(buf);

        stream.write_all(&public_key.to_be_bytes())?;
        stream.flush()?;
    } else {
        stream.write_all(&public_key.to_be_bytes())?;
        stream.flush()?;

        let mut buf = [0u8; 8];
        stream.read_exact(&mut buf)?;
        their_public_key = u64::from_be_bytes(buf);
    }

    println!("\n[DH] Computing shared secret...");
    let shared_secret = mod_exp(their_public_key, private_key, P);
    println!("     secret = {:016X}", shared_secret);

    Ok(shared_secret)
}

fn handle_chat(mut stream: TcpStream, shared_secret: u64) -> std::io::Result<()> {
    println!("\nSecure channel established!");
    println!("Type your messages below (Ctrl+C to quit):\n");

    let mut read_stream = stream.try_clone()?;
    let (tx, rx) = mpsc::channel::<()>();

    let read_handle = thread::spawn(move || {
        let mut reader = BufReader::new(&mut read_stream);
        let mut recv_keystream = KeystreamGenerator::new(shared_secret);

        loop {
            let mut len_buf = [0u8; 4];
            if reader.read_exact(&mut len_buf).is_err() {
                break;
            }
            let msg_len = u32::from_be_bytes(len_buf) as usize;

            if msg_len > 10000 {
                break;
            }

            let mut encrypted = vec![0u8; msg_len];
            if reader.read_exact(&mut encrypted).is_err() {
                break;
            }

            let decrypted = xor_cipher(&encrypted, &mut recv_keystream);
            if let Ok(message) = String::from_utf8(decrypted) {
                println!("\r[RECV] {}", message);
                print!("> ");
                std::io::stdout().flush().ok();
            }
        }
        let _ = tx.send(());
    });

    let stdin = std::io::stdin();
    let mut send_keystream = KeystreamGenerator::new(shared_secret);

    print!("> ");
    std::io::stdout().flush()?;

    for line in stdin.lock().lines() {
        if rx.try_recv().is_ok() {
            println!("\nConnection closed.");
            break;
        }

        let line = line?;
        if line.is_empty() {
            print!("> ");
            std::io::stdout().flush()?;
            continue;
        }

        let encrypted = xor_cipher(line.as_bytes(), &mut send_keystream);

        let len = encrypted.len() as u32;
        stream.write_all(&len.to_be_bytes())?;
        stream.write_all(&encrypted)?;
        stream.flush()?;

        println!("[SENT] {}", line);
        print!("> ");
        std::io::stdout().flush()?;
    }

    drop(stream);
    let _ = read_handle.join();
    Ok(())
}

fn run_server(port: u16) -> std::io::Result<()> {
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port))?;
    println!("[SERVER] Listening on 0.0.0.0:{}", port);

    let (mut stream, addr) = listener.accept()?;
    println!("[CLIENT] Connected from {}", addr);

    let shared_secret = perform_dh_exchange(&mut stream, true)?;

    handle_chat(stream, shared_secret)?;

    Ok(())
}

fn run_client(address: &str) -> std::io::Result<()> {
    println!("[CLIENT] Connecting to {}...", address);
    let mut stream = TcpStream::connect(address)?;
    println!("[CLIENT] Connected!");

    let shared_secret = perform_dh_exchange(&mut stream, false)?;

    handle_chat(stream, shared_secret)?;

    Ok(())
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Server { port } => run_server(port),
        Commands::Client { address } => run_client(&address),
    };

    if let Err(e) = result {
        eprintln!("[ERROR] {}", e);
        std::process::exit(1);
    }
}
