#![allow(dead_code)]
mod utils;
mod transform;
mod kernel;
mod nex_io;
mod whitebox;
mod quantum;
mod integrity;
mod compression;
mod kms;
mod audit;
mod standards;
mod protocol;
mod security;

use std::env;
use std::io::{self, Write};
use kernel::NexKernel;

// --- API Colors & styles ---
const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const CYAN: &str = "\x1b[36m";
const D_CYAN: &str = "\x1b[38;5;31m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const MAGENTA: &str = "\x1b[35m";
const GREY: &str = "\x1b[90m";
const RED: &str = "\x1b[31m";

fn print_banner() {
    println!("{}", D_CYAN);
    println!("███╗   ██╗███████╗██╗  ██╗    ██████╗ ██╗  ██╗██████╗  ██████╗ ");
    println!("████╗  ██║██╔════╝╚██╗██╔╝    ╚════██╗██║  ██║╚════██╗██╔════╝ ");
    println!("██╔██╗ ██║█████╗   ╚███╔╝█████╗██████╔╝███████║ █████╔╝███████╗");
    println!("██║╚██╗██║██╔══╝   ██╔██╗╚════╝██╔═══╝ ╚════██║██╔═══╝ ██╔═══██╗");
    println!("██║ ╚████║███████╗██╔╝ ██╗     ███████╗     ██║███████╗╚██████╔╝");
    println!("╚═╝  ╚═══╝╚══════╝╚═╝  ╚═╝     ╚══════╝     ╚═╝╚══════╝ ╚═════╝ ");
    println!("      >>> QUANTUM-RESISTANT CHAOS ENCRYPTION ENGINE <<<{}\n", RESET);
}

fn print_section(title: &str) {
    println!("\n{}═══ [ {} ] ════════════════════════════════════════════{}", CYAN, title, RESET);
}

fn print_kv(key: &str, val: &str) {
    println!("  {}{} : {}{}", GREY, key, RESET, val);
}

fn main() {
    print_banner();
    let args: Vec<String> = env::args().collect();

    // --- Benchmark Mode ---
    if args.len() > 1 && args[1] == "--bench" {
        print_section("BENCHMARK MODE");
        let cost = if args.len() > 2 { args[2].parse().unwrap_or(1) } else { 1 };
        
        let kernel = NexKernel::new(cost);
        println!("  {}• System   : {}AVX2 Vectorized + AES-NI Hardware Core", json_arrow(), GREEN);
        println!("  {}• Memory   : {}8 MB / Hash", json_arrow(), YELLOW);
        println!("  {}• Threads  : {}Auto-Scaling (All Cores)", json_arrow(), MAGENTA);
        println!();
        
        let hashes = kernel.benchmark();
        
        println!("\n  {}{}[ RESULT ] : {} {} Hashes/Second (Cost: {}){}", 
                 BOLD, GREEN, RESET, hashes, cost, RESET);
        return;
    }

    // --- Interactive Mode ---
    if args.len() <= 1 {
        print_section("INTERACTIVE SHELL");
        println!("  Type '{}exit{}' to quit.", YELLOW, RESET);
        let mut kernel = NexKernel::new(1); 
        kernel.enable_temporal_binding(); 
        
        loop {
            let mut input_buffer = String::new();
            print!("\n{}nex24{} > ", CYAN, RESET);
            io::stdout().flush().unwrap();
            
            if io::stdin().read_line(&mut input_buffer).is_err() { break; }
            let input_str = input_buffer.trim();
            
            if input_str == "exit" { break; }
            if input_str.is_empty() { continue; }
            
            // Wrap String in Cursor for streaming interface
            let mut cursor = std::io::Cursor::new(input_str.as_bytes());
            let result = kernel.execute(&mut cursor, "InteractiveKey");
            println!("  {}Hash >> {}{}", GREEN, result.full_formatted_string, RESET);
        }
        return;
    }

    // --- File Encryption Mode ---
    if args.len() > 1 && args[1] == "--encrypt" {
        if args.len() < 4 {
            println!("Usage: nex2426 --encrypt <file> <key> [cost] [--bio-lock] [--stealth]");
            return;
        }
        let input_path = &args[2];
        let key = &args[3];
        let mut cost = 1;
        
        // Parse optional args
        let mut use_bio_lock = false;
        let mut is_stealth = false;
        
        for i in 4..args.len() {
            if args[i] == "--bio-lock" {
                use_bio_lock = true;
            } else if args[i] == "--stealth" {
                is_stealth = true;
            } else if let Ok(c) = args[i].parse::<u32>() {
                cost = c;
            }
        }
        
        let output_path = format!("{}.nex2426", input_path);

        print_section("FILE ENCRYPTION MODE");
        print_kv("Input File", input_path);
        print_kv("Output File", &output_path);
        print_kv("Key Strength", &format!("Cost {}", cost));
        if use_bio_lock {
            print_kv("Security", &format!("{}BIO-LOCKED{}", RED, RESET));
        }
        if is_stealth {
            print_kv("Mode", &format!("{}STEALTH (No Header){}", MAGENTA, RESET));
        }

        println!("\n  {}Encrypting...{}", YELLOW, RESET);
        let start = std::time::Instant::now();
        
        match utils::file_ops::encrypt_file(input_path, &output_path, key, cost, use_bio_lock, is_stealth) {
            Ok(_) => {
                println!("  {}Success! File encrypted securely.{}", GREEN, RESET);
                println!("  {}Time: {:?}{}", GREY, start.elapsed(), RESET);
            },
            Err(e) => println!("{}Error: {}{}", RED, e, RESET),
        }
        return;
    }

    // --- File Decryption Mode ---
    if args.len() > 1 && args[1] == "--decrypt" {
        if args.len() < 4 {
            println!("Usage: nex2426 --decrypt <file.nex2426> <key> [--stealth]");
            return;
        }
        let input_path = &args[2];
        let key = &args[3];
        let is_stealth = args.iter().any(|a| a == "--stealth");
        
        // Remove .nex2426 extension or append .decrypted
        let output_path = if input_path.ends_with(".nex2426") {
            input_path.replace(".nex2426", "")
        } else {
            format!("{}.decrypted", input_path)
        };

        print_section("FILE DECRYPTION MODE");
        print_kv("Encrypted File", input_path);
        print_kv("Output File", &output_path);
        if is_stealth {
            print_kv("Mode", &format!("{}STEALTH MODE DECRYPTION{}", MAGENTA, RESET));
        }

        println!("\n  {}Decrypting...{}", YELLOW, RESET);
        let start = std::time::Instant::now();

        match utils::file_ops::decrypt_file(input_path, &output_path, key, is_stealth) {
            Ok(_) => {
                println!("  {}Success! File decrypted.{}", GREEN, RESET);
                println!("  {}Time: {:?}{}", GREY, start.elapsed(), RESET);
            },
            Err(e) => println!("{}Error: {}{}", RED, e, RESET),
        }
        return;
    }

    // --- Sign & Verify Modes ---
    if args.len() > 1 && args[1] == "--sign" {
       println!("{}Generating Ephemeral Keypair for Signing...{}", YELLOW, RESET);
       let signer = crate::standards::signatures::NexSigner::new();
       let (sk, _pk) = signer.generate_keypair();
       
       let msg = if args.len() > 2 { args[2].as_bytes() } else { b"Test Message" };
       let sig = signer.sign(msg, &sk);
       
       println!("{}Signature Generated!{}", GREEN, RESET);
       println!("Message: {:?}", String::from_utf8_lossy(msg));
       println!("Challenge (c): {:?}", sig.c);
       println!("Response Vector (z): [{} coeffs]", sig.z.len());
       return;
    }
    
    if args.len() > 1 && args[1] == "--blockchain" {
        print_section("NEX LEDGER DEMO");
        use crate::integrity::ledger::NexLedger;
        use crate::standards::signatures::NexSigner;
        
        println!("{}Initializing Quantum Ledger...{}", YELLOW, RESET);
        let mut ledger = NexLedger::new();
        let signer = NexSigner::new();
        let (sk, pk) = signer.generate_keypair();
        
        println!("  {}Genesis Block Hash: {}{}", GREY, ledger.chain[0].hash, RESET);
        
        // Block 1
        println!("\n{}Mining Block 1...{}", CYAN, RESET);
        let tx1 = vec![b"UserA -> UserB: 100 NEX".as_slice()];
        let b1 = ledger.create_block(tx1, ledger.chain.last().unwrap().hash.clone(), 1, &sk);
        
        if ledger.add_block(b1.clone(), &pk) {
            println!("  {}Block 1 Added [Hash: {}...]{}", GREEN, &b1.hash[0..16], RESET);
        } else {
            println!("  {}Block 1 Failed!{}", RED, RESET);
        }

        // Block 2
        println!("\n{}Mining Block 2...{}", CYAN, RESET);
        let tx2 = vec![b"UserC -> UserA: 50 NEX".as_slice()];
        let b2 = ledger.create_block(tx2, ledger.chain.last().unwrap().hash.clone(), 2, &sk);
        
        if ledger.add_block(b2.clone(), &pk) {
            println!("  {}Block 2 Added [Hash: {}...]{}", GREEN, &b2.hash[0..16], RESET);
        } 
        
        // Tamper Attempt
        println!("\n{}Attempting Tampered Block...{}", RED, RESET);
        let tx3 = vec![b"Attack -> Hacker: 1M NEX".as_slice()];
        let mut b3 = ledger.create_block(tx3, ledger.chain.last().unwrap().hash.clone(), 3, &sk);
        b3.transactions_root = "FAKE_ROOT".to_string(); // Tamper with content linkage
        
        if ledger.add_block(b3, &pk) {
            println!("  {}Tampered Block Accepted (FAIL!){}", RED, RESET);
        } else {
             println!("  {}Tampered Block Rejected (SUCCESS!){}", GREEN, RESET);
        }

        return;
    }

    // --- Standard CLI Mode (Hashing) ---
    // ... (Existing logic)
    
    let mut key = "SecretKey123";
    let mut cost = 1;
    let mut input_reader: Box<dyn std::io::Read>;
    let mut input_desc = "Raw String";

    if args[1] == "--file" {
        if args.len() < 3 {
            println!("{}Error: Missing file path after --file{}", RED, RESET);
            return;
        }
        let path = &args[2];
        match std::fs::File::open(path) {
            Ok(f) => {
                input_reader = Box::new(std::io::BufReader::new(f));
                input_desc = "File Stream";
                if args.len() > 3 { key = &args[3]; }
                if args.len() > 4 { cost = args[4].parse().unwrap_or(1); }
            }
            Err(e) => {
                println!("{}Error reading file: {}{}", RED, e, RESET);
                return;
            }
        }
    } else {
        // Standard String Input
        let input_str = &args[1];
        input_reader = Box::new(std::io::Cursor::new(input_str.as_bytes()));
        if args.len() > 2 { key = &args[2]; }
        if args.len() > 3 { cost = args[3].parse().unwrap_or(1); }
    }

    print_section("CONFIGURATION");
    print_kv("Input Type", input_desc);
    print_kv("Secret Key", key);
    print_kv("Difficulty", &format!("{} (Memory Hardened)", cost));
    
    // Execution
    let mut kernel = NexKernel::new(cost);
    kernel.enable_temporal_binding();
    println!("\n  {}Processing...{}", YELLOW, RESET);
    let start = std::time::Instant::now();
    
    let result = kernel.execute(&mut input_reader, key);
    
    let duration = start.elapsed();

    // Output
    print_section("OUTPUT");
    println!("  {}Execution Time : {}{:?}{}", GREY, YELLOW, duration, RESET);
    println!("  {}Timestamp      : {}{}{}", GREY, CYAN, result.timestamp, RESET);
    
    println!("\n  {}► ENCRYPTED SIGNATURE (Standard):{}", BOLD, RESET);
    println!("  {}{}", GREEN, result.full_formatted_string);
    
    println!("\n  {}► COMPRESSED (Base58):{}", BOLD, RESET);
    println!("  {}{}{}", CYAN, result.hash_base58, RESET);
    
    println!();
}

fn json_arrow() -> &'static str {
    "\x1b[36m➜\x1b[0m "
}

