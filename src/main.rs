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
mod blockchain;
mod hardware;
mod error;
mod validation;

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
    println!("      >>> SECURE DATA STORAGE ENCRYPTION ENGINE <<<{}\n", RESET);
}

fn print_section(title: &str) {
    println!("\n{}═══ [ {} ] ════════════════════════════════════════════{}", CYAN, title, RESET);
}

fn print_kv(key: &str, val: &str) {
    println!("  {}{} : {}{}", GREY, key, RESET, val);
}

fn print_help() {
    println!("{}", CYAN);
    println!("███╗   ██╗███████╗██╗  ██╗    ██████╗ ██╗  ██╗██████╗  ██████╗ ");
    println!("████╗  ██║██╔════╝╚██╗██╔╝    ╚════██╗██║  ██║╚════██╗██╔════╝ ");
    println!("██╔██╗ ██║█████╗   ╚███╔╝█████╗██████╔╝███████║ █████╔╝███████╗");
    println!("██║╚██╗██║██╔══╝   ██╔██╗╚════╝██╔═══╝ ╚════██║██╔═══╝ ██╔═══██╗");
    println!("██║ ╚████║███████╗██╔╝ ██╗     ███████╗     ██║███████╗╚██████╔╝");
    println!("╚═╝  ╚═══╝╚══════╝╚═╝  ╚═╝     ╚══════╝     ╚═╝╚══════╝ ╚═════╝ ");
    println!("      >>> SECURE DATA STORAGE ENCRYPTION ENGINE <<<{}\n", RESET);
    
    println!("{}USAGE:\n", BOLD);
    println!("  {}nex2426 [OPTIONS] [INPUT] [KEY] [COST]\n", CYAN);
    
    println!("{}MODES:\n", BOLD);
    
    println!("{}Interactive Mode:\n", YELLOW);
    println!("  {}nex2426                           # Start interactive shell", CYAN);
    println!("                                     # Type 'exit' to quit\n");
    
    println!("{}Help:\n", YELLOW);
    println!("  {}--help, -h                       # Show this help message\n", CYAN);
    
    println!("{}Benchmark Mode:\n", YELLOW);
    println!("  {}--bench [cost]                   # Performance benchmark", CYAN);
    println!("                                     # Default cost: 1\n");
    
    println!("{}File Encryption:\n", YELLOW);
    println!("  {}--encrypt <file> <key> [cost] [--bio-lock] [--stealth]", CYAN);
    println!("                                     # Encrypt file with NEX2426");
    println!("                                     # --bio-lock: Hardware binding");
    println!("                                     # --stealth: No header mode\n");
    
    println!("{}File Decryption:\n", YELLOW);
    println!("  {}--decrypt <file.nex2426> <key> [--stealth]", CYAN);
    println!("                                     # Decrypt encrypted file\n");
    
    println!("{}Digital Signature:\n", YELLOW);
    println!("  {}--sign [message]                 # Generate signature", CYAN);
    println!("                                     # Default: \"Test Message\"\n");
    
    println!("{}Blockchain Demo:\n", YELLOW);
    println!("  {}--blockchain                     # Run blockchain demo", CYAN);
    println!("                                     # Shows mining & tamper detection\n");
    
    println!("{}Hardware Acceleration:\n", YELLOW);
    println!("  {}--hardware [cost]                 # Use hardware acceleration", CYAN);
    println!("                                     # Requires FPGA/ASIC support");
    println!("  {}--hw-test                        # Run hardware tests", CYAN);
    println!("  {}--hw-bench [cost]                # Hardware benchmark", CYAN);
    println!("                                     # Compare software vs hardware\n");
    
    println!("{}Hashing Modes:\n", YELLOW);
    println!("  {}--file <path> [key] [cost]       # Hash file content", CYAN);
    println!("  {}<string> [key] [cost]            # Hash string input", CYAN);
    println!("                                     # Default key: [user-provided or generated]");
    println!("                                     # Default cost: 1\n");
    
    println!("{}EXAMPLES:\n", BOLD);
    println!("  {}nex2426 --help                   # Show help", CYAN);
    println!("  {}nex2426 --bench 2                # Benchmark with cost 2", CYAN);
    println!("  {}nex2426 --encrypt data.txt mykey  # Encrypt file", CYAN);
    println!("  {}nex2426 --decrypt data.txt.nex2426 mykey  # Decrypt file", CYAN);
    println!("  {}nex2426 --file document.pdf      # Hash file", CYAN);
    println!("  {}nex2426 \"Hello World\" mykey 3   # Hash string with custom key/cost", CYAN);
    println!("  {}nex2426                           # Interactive mode\n", CYAN);
    
    println!("{}OUTPUT FORMAT:\n", BOLD);
    println!("  The program outputs encrypted signatures in the following format:");
    println!("  {}$nex6$v=<version>$c=<compression>$t=<timestamp>$s=<salt>$<hash>$", GREEN);
    println!("  And Base58 compressed format for easy sharing.\n");
    
    println!("{}SECURITY FEATURES:\n", BOLD);
    println!("  • Quantum-resistant chaos encryption");
    println!("  • Memory-hardened hashing (Argon2-inspired)");
    println!("  • Hardware binding (Bio-Lock)");
    println!("  • Stealth mode (no metadata headers)");
    println!("  • Temporal binding for enhanced security\n");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    // --- Help Mode ---
    if args.len() > 1 && (args[1] == "--help" || args[1] == "-h") {
        print_help();
        return;
    }

    print_banner();

    // --- Benchmark Mode ---
    if args.len() > 1 && args[1] == "--bench" {
        print_section("BENCHMARK MODE");
        let cost = if args.len() > 2 {
            match args[2].parse::<u32>() {
                Ok(c) if c > 0 && c <= 1000 => c,
                Ok(_) => {
                    eprintln!("{}Error: Cost must be between 1 and 1000, using default 1{}", RED, RESET);
                    1
                },
                Err(_) => {
                    eprintln!("{}Error: Invalid cost format, using default 1{}", RED, RESET);
                    1
                }
            }
        } else { 1 };
        
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

    // --- Hardware Acceleration Mode ---
    if args.len() > 1 && (args[1] == "--hardware" || args[1] == "--hw-test" || args[1] == "--hw-bench") {
        use crate::hardware::HardwareAccelerator;
        
        let cost = if args.len() > 2 && args[1] != "--hw-test" {
            match args[2].parse::<u32>() {
                Ok(c) if c > 0 && c <= 1000 => c,
                Ok(_) => {
                    eprintln!("{}Error: Cost must be between 1 and 1000, using default 1{}", RED, RESET);
                    1
                },
                Err(_) => {
                    eprintln!("{}Error: Invalid cost format, using default 1{}", RED, RESET);
                    1
                }
            }
        } else { 1 };
        
        if args[1] == "--hw-test" {
            print_section("HARDWARE TEST MODE");
            let accelerator = HardwareAccelerator::new(5000);
            
            println!("  {}Initializing hardware bridge...{}", YELLOW, RESET);
            match accelerator.reset() {
                Ok(_) => println!("  {}Hardware bridge initialized successfully{}", GREEN, RESET),
                Err(e) => {
                    eprintln!("  {}Hardware bridge initialization failed: {}{}", RED, e, RESET);
                    return;
                }
            }
            
            println!("  {}Running comprehensive hardware tests...{}", YELLOW, RESET);
            match accelerator.run_comprehensive_test() {
                Ok(results) => {
                    println!("\n  {}{}[ TEST RESULTS ]{}", BOLD, CYAN, RESET);
                    println!("  {}Hash Test        : {}{}{}", GREY, if results.hash_test_passed { GREEN } else { RED }, 
                            if results.hash_test_passed { "PASSED" } else { "FAILED" }, RESET);
                    println!("  {}Hash Time        : {}{:.2}ms{}", GREY, YELLOW, results.hash_time.as_millis() as f64, RESET);
                    if let Some(ref hash) = results.hash_result {
                        println!("  {}Hash Result      : {}{:.16}...{}", GREY, CYAN, hash, RESET);
                    }
                    
                    println!("  {}Encryption Test   : {}{}{}", GREY, if results.encryption_test_passed { GREEN } else { RED }, 
                            if results.encryption_test_passed { "PASSED" } else { "FAILED" }, RESET);
                    println!("  {}Encrypt Time      : {}{:.2}ms{}", GREY, YELLOW, results.encrypt_time.as_millis() as f64, RESET);
                    println!("  {}Decrypt Time      : {}{:.2}ms{}", GREY, YELLOW, results.decrypt_time.as_millis() as f64, RESET);
                    
                    println!("  {}Benchmark Time    : {}{:.2}ms{}", GREY, YELLOW, results.benchmark_time.as_millis() as f64, RESET);
                    println!("  {}Benchmark Cycles  : {}{}{}", GREY, MAGENTA, results.benchmark_cycles, RESET);
                    
                    if let Some(ref metrics) = results.performance_metrics {
                        println!("  {}Hardware Version  : {}{}.{}{}", GREY, GREEN, metrics.version_major, metrics.version_minor, RESET);
                        println!("  {}Hardware Status   : {}{:?}{}", GREY, YELLOW, metrics.current_status, RESET);
                        println!("  {}Hardware Ready    : {}{}{}", GREY, if metrics.is_ready { GREEN } else { RED }, 
                                if metrics.is_ready { "YES" } else { "NO" }, RESET);
                    }
                    
                    if results.all_tests_passed() {
                        println!("\n  {}{}[ ALL TESTS PASSED ]{}", BOLD, GREEN, RESET);
                    } else {
                        println!("\n  {}{}[ SOME TESTS FAILED ]{}", BOLD, RED, RESET);
                    }
                },
                Err(e) => {
                    eprintln!("  {}Hardware test failed: {}{}", RED, e, RESET);
                }
            }
        } else if args[1] == "--hw-bench" {
            print_section("HARDWARE BENCHMARK MODE");
            let accelerator = HardwareAccelerator::new(5000);
            
            println!("  {}Running hardware benchmark...{}", YELLOW, RESET);
            match accelerator.benchmark(cost) {
                Ok((duration, cycles)) => {
                    println!("  {}Hardware Time     : {}{:.2}ms{}", GREY, YELLOW, duration.as_millis() as f64, RESET);
                    println!("  {}Hardware Cycles   : {}{}{}", GREY, MAGENTA, cycles, RESET);
                    println!("  {}Hardware Throughput: {}{:.2} ops/sec{}", GREY, GREEN, 
                            1000.0 / duration.as_secs_f64(), RESET);
                    
                    // Compare with software
                    println!("\n  {}Comparing with software implementation...{}", YELLOW, RESET);
                    let kernel = NexKernel::new(cost);
                    let software_hashes = kernel.benchmark();
                    let hardware_ops = 1000.0 / duration.as_secs_f64();
                    
                    println!("  {}Software Throughput: {}{} hashes/sec{}", GREY, CYAN, software_hashes, RESET);
                    println!("  {}Hardware Throughput: {}{:.2} ops/sec{}", GREY, GREEN, hardware_ops, RESET);
                    
                    if hardware_ops > software_hashes as f64 {
                        let speedup = hardware_ops / software_hashes as f64;
                        println!("  {}Speedup           : {}{:.2}x faster{}", GREY, GREEN, speedup, RESET);
                    } else {
                        let slowdown = software_hashes as f64 / hardware_ops;
                        println!("  {}Slowdown         : {}{:.2}x slower{}", GREY, YELLOW, slowdown, RESET);
                    }
                },
                Err(e) => {
                    eprintln!("  {}Hardware benchmark failed: {}{}", RED, e, RESET);
                }
            }
        } else {
            print_section("HARDWARE ACCELERATION MODE");
            let accelerator = HardwareAccelerator::new(5000);
            
            println!("  {}Using hardware acceleration with cost {}{}", YELLOW, cost, RESET);
            println!("  {}Hardware bridge initialized{}", GREEN, RESET);
            
            // For demonstration, hash a test string
            let test_data = b"Hardware Accelerated Test";
            let test_key = [0x48, 0x57, 0x41, 0x43, 0x43, 0x45, 0x4C, 0x45, 0x52, 0x41, 0x54, 0x45, 0x44, 0x54, 0x45, 0x53,
                            0x54, 0x44, 0x41, 0x54, 0x41, 0x48, 0x57, 0x41, 0x43, 0x43, 0x45, 0x4C, 0x45, 0x52, 0x41, 0x54];
            
            match accelerator.hash_data(test_data, &test_key, cost) {
                Ok(hash) => {
                    println!("  {}Hardware Hash Result:{}", GREEN, RESET);
                    println!("  {}{}{}", CYAN, hex::encode(hash), RESET);
                    
                    // Compare with software
                    let mut cursor = std::io::Cursor::new(test_data);
                    let kernel = NexKernel::new(cost);
                    let software_result = kernel.execute(&mut cursor, "HardwareTest");
                    
                    println!("\n  {}Software Hash Result:{}", GREEN, RESET);
                    println!("  {}{}{}", CYAN, software_result.full_formatted_string, RESET);
                    
                    println!("\n  {}Hardware acceleration completed successfully!{}", GREEN, RESET);
                },
                Err(e) => {
                    eprintln!("  {}Hardware acceleration failed: {}{}", RED, e, RESET);
                }
            }
        }
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
            if let Err(e) = io::stdout().flush() {
                eprintln!("{}Error flushing output: {}{}", RED, e, RESET);
                break;
            }
            
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
            eprintln!("{}Error: Missing arguments{}", RED, RESET);
            println!("Usage: nex2426 --encrypt <file> <key> [cost] [--bio-lock] [--stealth]");
            return;
        }
        let input_path = &args[2];
        let key = &args[3];
        
        // Validate inputs
        if key.len() < 8 {
            eprintln!("{}Error: Key must be at least 8 characters long{}", RED, RESET);
            return;
        }
        
        if !std::path::Path::new(input_path).exists() {
            eprintln!("{}Error: Input file does not exist: {}{}", RED, input_path, RESET);
            return;
        }
        
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
                if c > 0 && c <= 1000 {
                    cost = c;
                } else {
                    eprintln!("{}Error: Cost must be between 1 and 1000, using default 1{}", RED, RESET);
                }
            } else {
                eprintln!("{}Warning: Unknown argument '{}', ignoring{}", RED, args[i], RESET);
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
        let last_hash = ledger.chain.last()
            .map(|block| block.hash.clone())
            .unwrap_or_else(|| {
                eprintln!("{}Error: No genesis block found{}", RED, RESET);
                std::process::exit(1);
            });
        let b1 = ledger.create_block(tx1, last_hash, 1, &sk);
        
        if ledger.add_block(b1.clone(), &pk) {
            println!("  {}Block 1 Added [Hash: {}...]{}", GREEN, &b1.hash[0..16], RESET);
        } else {
            println!("  {}Block 1 Failed!{}", RED, RESET);
        }

        // Block 2
        println!("\n{}Mining Block 2...{}", CYAN, RESET);
        let tx2 = vec![b"UserC -> UserA: 50 NEX".as_slice()];
        let last_hash = ledger.chain.last()
            .map(|block| block.hash.clone())
            .unwrap_or_else(|| {
                eprintln!("{}Error: No previous block found{}", RED, RESET);
                std::process::exit(1);
            });
        let b2 = ledger.create_block(tx2, last_hash, 2, &sk);
        
        if ledger.add_block(b2.clone(), &pk) {
            println!("  {}Block 2 Added [Hash: {}...]{}", GREEN, &b2.hash[0..16], RESET);
        } 
        
        // Tamper Attempt
        println!("\n{}Attempting Tampered Block...{}", RED, RESET);
        let tx3 = vec![b"Attack -> Hacker: 1M NEX".as_slice()];
        let last_hash = ledger.chain.last()
            .map(|block| block.hash.clone())
            .unwrap_or_else(|| {
                eprintln!("{}Error: No previous block found{}", RED, RESET);
                std::process::exit(1);
            });
        let mut b3 = ledger.create_block(tx3, last_hash, 3, &sk);
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
    
    let mut key = ""; // Will be set below or remain empty for error
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
                if args.len() > 4 { 
                    cost = match args[4].parse::<u32>() {
                        Ok(c) if c > 0 && c <= 1000 => c,
                        Ok(_) => {
                            eprintln!("{}Error: Cost must be between 1 and 1000, using default 1{}", RED, RESET);
                            1
                        },
                        Err(_) => {
                            eprintln!("{}Error: Invalid cost format, using default 1{}", RED, RESET);
                            1
                        }
                    };
                }
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
        if args.len() > 3 { 
            cost = match args[3].parse::<u32>() {
                Ok(c) if c > 0 && c <= 1000 => c,
                Ok(_) => {
                    eprintln!("{}Error: Cost must be between 1 and 1000, using default 1{}", RED, RESET);
                    1
                },
                Err(_) => {
                    eprintln!("{}Error: Invalid cost format, using default 1{}", RED, RESET);
                    1
                }
            };
        }
    }

    // Generate secure random key if none provided
    let final_key = if key.is_empty() {
        use crate::utils::entropy::SecureRng;
        let mut rng = SecureRng::new().unwrap_or_else(|_| {
            eprintln!("{}Warning: Using fallback key generation{}", YELLOW, RESET);
            SecureRng::default()
        });
        let mut key_bytes = [0u8; 16];
        rng.fill_bytes(&mut key_bytes).unwrap_or_else(|_| {
            // Fallback to timestamp-based key
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            key_bytes.copy_from_slice(&timestamp.to_be_bytes());
        });
        hex::encode(key_bytes)[..16].to_string()
    } else {
        key.to_string()
    };
    
    print_section("CONFIGURATION");
    print_kv("Input Type", input_desc);
    let display_key = if final_key.len() > 16 { 
        format!("{}...", &final_key[..16]) 
    } else { 
        final_key.clone() 
    };
    print_kv("Secret Key", &display_key);
    print_kv("Difficulty", &format!("{} (Memory Hardened)", cost));
    
    // Execution
    let mut kernel = NexKernel::new(cost);
    kernel.enable_temporal_binding();
    println!("\n  {}Processing...{}", YELLOW, RESET);
    let start = std::time::Instant::now();
    
    let result = kernel.execute(&mut input_reader, &final_key);
    
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

