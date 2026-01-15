use clap::{Parser, Subcommand};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use rand::Rng;

const P: u64 = 0xD87FA3E291B4C7F3;
const G: u64 = 2;

#[derive(Parser)]
#[command(name = "streamchat")]
#[command(about = "Stream cipher chat with Diffie-Hellman key generation")]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Server { port: u16 },
    Client { address: String },
}

fn exp_mod(b: u64, e: u64, m: u64) -> u64 {
    let mut res = 1u64;
    let mut base = b % m;
    let mut exp = e;

    loop {
        if exp == 0 {
            break;
        }
        if exp & 1 == 1 {
            res = ((res as u128 * base as u128) % m as u128) as u64;
        }
        exp = exp >> 1;
        base = ((base as u128 * base as u128) % m as u128) as u64;
    }

    res
}

fn make_pub_key(priv_key: u64) -> u64 {
    exp_mod(G, priv_key, P)
}

fn make_secret(priv_key: u64, other_pub: u64) -> u64 {
    exp_mod(other_pub, priv_key, P)
}

fn start_server(port: u16) {
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).unwrap();
    println!("[SERVER] Listening on 0.0.0.0:{}", port);

    for incoming in listener.incoming() {
        let mut socket = incoming.unwrap();
        let addr = socket.peer_addr().unwrap();
        println!("[CLIENT] Connected from {}", addr);

        println!("[DH] Starting key exchange...");
        println!("[DH] Using hardcoded DH parameters:");
        println!("p = {:X} (64-bit prime - public)", P);
        println!("g = {} (generator - public)", G);

        let mut rng = rand::thread_rng();
        let priv = rng.gen::<u64>();
        let pub_key = make_pub_key(priv);

        println!("\n[DH] Generating our keypair...");
        println!("private_key = {:X} (random 64-bit)", priv);
        println!("public_key  = g^private mod p = {:X}", pub_key);

        let mut data = [0u8; 8];
        socket.write_all(&pub_key.to_le_bytes()).unwrap();
        socket.read_exact(&mut data).unwrap();
        let other_pub = u64::from_le_bytes(data);

        let secret = make_secret(priv, other_pub);
        println!("\n[DH] Computing shared secret...");
        println!("secret = {:X}", secret);
        println!("\nSecure channel established!\n");
    }
}

fn start_client(addr: &str) {
    let mut socket = TcpStream::connect(addr).unwrap();
    println!("[CLIENT] Connecting to {}...", addr);
    println!("[CLIENT] Connected!");

    println!("\n[DH] Starting key exchange...");

    let mut rng = rand::thread_rng();
    let priv = rng.gen::<u64>();
    let pub_key = make_pub_key(priv);

    let mut data = [0u8; 8];
    socket.read_exact(&mut data).unwrap();
    let other_pub = u64::from_le_bytes(data);

    socket.write_all(&pub_key.to_le_bytes()).unwrap();

    let secret = make_secret(priv, other_pub);
    println!("[DH] Computing shared secret...");
    println!("secret = {:X}", secret);
    println!("\nSecure channel established!\n");
}

fn main() {
    let args = Args::parse();

    match args.command {
        Commands::Server { port } => start_server(port),
        Commands::Client { address } => start_client(&address),
    }
}
