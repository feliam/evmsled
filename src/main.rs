//! EVM Function Dispatcher Generator
//! 
//! This is a proof of concept for generating optimized EVM function dispatchers.
//! It uses a mathematical approach to map function selectors to unique bytes,
//! which are then used to create a jump table for function dispatching.
//!
//! # How it works
//! 1. Generate random function selectors (simulating keccak256 hashes)
//! 2. Find magic numbers (q and shift) that map selectors to unique bytes
//! 3. Generate bytecode for the dispatcher and function jumps
//!
//! # Collision Detection
//! The core of this dispatcher is finding magic numbers (q and shift) that map each
//! function selector to a unique byte. This is done by:
//! 1. Taking a function selector (4 bytes)
//! 2. Multiplying it by a magic number q
//! 3. Shifting right by shift bits
//! 4. Taking the lowest byte
//! If any two selectors map to the same byte, we have a collision and need to try
//! different magic numbers. The goal is to find magic numbers that produce no collisions
//! while keeping the maximum byte value as low as possible.
//!
//! # Suggestions for improvement
//! - Add proper error handling instead of using Option
//! - Add tests for the mathematical operations
//! - Add benchmarks to compare with other dispatcher implementations
//! - Add support for custom function selectors
//! - Add validation for magic number quality
//! - Add support for different bytecode layouts
//! - Add support for different address spaces

use rand::Rng;
use rand::distributions::Standard;
use std::collections::HashSet;
use std::time::Instant;
use std::collections::HashMap;

/// Simulates EVM operations for function dispatching:
/// 1. CALLDATALOAD(0) gets first 32 bytes of calldata
/// 2. AND(0xFFFFFFFF) keeps only first 4 bytes (function selector)
/// 3. MUL by magic number q
/// 4. SHR by magic number shift
/// 5. AND(0xFF) gets lowest byte
/// This byte will be unique for each function selector
/// Then we can use this byte to jump to the correct function

/// Generates a random 256-bit integer as [u8; 32]
fn random_256bit() -> [u8; 32] {
    rand::thread_rng().sample(Standard)
}

/// Converts u32 to [u8; 32] (little endian)
/// This simulates the first 4 bytes of CALLDATALOAD(0)
fn u32_to_256(x: u32) -> [u8; 32] {
    let mut arr = [0u8; 32];
    arr[..4].copy_from_slice(&x.to_le_bytes());
    arr
}

/// Multiplies two 256-bit numbers (as [u8; 32]), returns lower 32 bytes (mod 2^256)
/// This simulates the MUL operation in EVM
fn mul_256(a: &[u8; 32], b: &[u8; 32]) -> [u8; 32] {
    let mut result = [0u8; 32];
    for i in 0..32 {
        let mut carry = 0u16;
        for j in 0..(32 - i) {
            let ai = a[i] as u16;
            let bj = b[j] as u16;
            let ri = result[i + j] as u16;
            let prod = ai * bj + ri + carry;
            result[i + j] = (prod & 0xFF) as u8;
            carry = prod >> 8;
        }
    }
    result
}

/// Shifts a 256-bit number right by n bits
/// This simulates the SHR operation in EVM
fn shr_256(val: &[u8; 32], n: u32) -> [u8; 32] {
    let mut result = [0u8; 32];
    let byte_shift = (n / 8) as usize;
    let bit_shift = n % 8;
    for i in 0..32 {
        if i + byte_shift < 32 {
            let mut v = val[i + byte_shift] >> bit_shift;
            if bit_shift > 0 && i + byte_shift + 1 < 32 {
                v |= val[i + byte_shift + 1] << (8 - bit_shift);
            }
            result[i] = v & 0xFF;
        }
    }
    result
}

/// Generates function selectors (first 4 bytes of keccak256(function signature))
fn generate_function_selectors(count: usize) -> Vec<u32> {
    let mut rng = rand::thread_rng();
    let mut values = HashSet::new();
    while values.len() < count {
        values.insert(rng.gen::<u32>());
    }
    values.into_iter().collect()
}

/// Checks if the given magic numbers (q and shift) produce unique bytes for all selectors
/// 
/// # Collision Detection
/// This function checks for collisions by:
/// 1. Converting each selector to a 256-bit number
/// 2. Multiplying by magic number q
/// 3. Shifting right by shift bits
/// 4. Taking the lowest byte
/// 5. Checking if this byte has been seen before
/// 
/// Returns false if any collision is found, true if all selectors map to unique bytes
fn check_magic_numbers(q: &[u8; 32], shift: u32, values: &[u32]) -> bool {
    let mut results = HashSet::new();
    for &x in values {
        let x256 = u32_to_256(x);
        let prod = mul_256(&x256, q);
        let shifted = shr_256(&prod, shift);
        let result_byte = shifted[0];
        if !results.insert(result_byte) {
            return false;
        }
    }
    true
}

/// Finds magic numbers that map all selectors to unique bytes
/// 
/// # Collision Avoidance
/// This function tries different combinations of q and shift until it finds one
/// that produces no collisions. For each attempt:
/// 1. Generates a random 256-bit number for q
/// 2. Tries different shift values (0 to 248, step 8)
/// 3. Checks for collisions using check_magic_numbers
/// 4. If no collisions, calculates the maximum byte value
/// 5. Keeps track of the solution with the lowest maximum byte value
/// 
/// Returns (q, shift) if found, None otherwise
fn find_magic_numbers(values: &[u32], max_attempts: u32) -> Option<([u8; 32], u32)> {
    let start_time = Instant::now();
    let mut attempts = 0;
    let mut best_solution: Option<([u8; 32], u32)> = None;
    let mut best_max_byte = 255u8;

    while attempts < max_attempts {
        let q = random_256bit();
        for shift in (0..=248).step_by(8) {
            if check_magic_numbers(&q, shift, values) {
                let mut max_byte = 0u8;
                for &x in values {
                    let x256 = u32_to_256(x);
                    let prod = mul_256(&x256, &q);
                    let shifted = shr_256(&prod, shift);
                    max_byte = max_byte.max(shifted[0]);
                }

                if max_byte < best_max_byte {
                    best_max_byte = max_byte;
                    best_solution = Some((q, shift));
                    let duration = start_time.elapsed();
                    println!("Found better solution after {} attempts in {:.2} seconds (max byte: 0x{:02x})", 
                            attempts, duration.as_secs_f64(), max_byte);
                }
            }
            attempts += 1;
        }
    }

    if let Some((q, shift)) = best_solution {
        let duration = start_time.elapsed();
        println!("Best solution found after {} attempts in {:.2} seconds (max byte: 0x{:02x})", 
                attempts, duration.as_secs_f64(), best_max_byte);
        Some((q, shift))
    } else {
        None
    }
}

/// Generates function addresses in the format 0xff1000, 0xff2000, etc.
fn generate_function_addresses(count: usize) -> Vec<u32> {
    (0..count).map(|i| 0xf00000 + ((i as u32 + 1) * 0x1000)).collect()
}

fn main() {
    // Generate function selectors (simulating first 4 bytes of keccak256(function signature))
    let function_selectors = generate_function_selectors(20);
    let function_addresses = generate_function_addresses(20);
    
    println!("Generated function selectors and addresses:");
    for (i, (&selector, &addr)) in function_selectors.iter().zip(function_addresses.iter()).enumerate() {
        println!("{:2}: Selector: 0x{:08x} -> Address: 0x{:08x}", i + 1, selector, addr);
    }
    
    match find_magic_numbers(&function_selectors, 1_000) {
        Some((q, shift)) => {
            println!("\nFound magic numbers for EVM dispatch:");
            print!("q (multiplier): 0x");
            for &b in q.iter().rev() { print!("{:02x}", b); }
            println!("");
            println!("shift: {}", shift);
            
            println!("\nSelector to Result Byte Mapping:");
            println!("--------------------------------");
            println!("Selector\t\tResult Byte");
            println!("--------------------------------");
            for &selector in &function_selectors {
                let x256 = u32_to_256(selector);
                let prod = mul_256(&x256, &q);
                let shifted = shr_256(&prod, shift);
                let result_byte = shifted[0];
                println!("0x{:08x}\t\t0x{:02x}", selector, result_byte);
            }
            println!("--------------------------------\n");
            
            println!("\nEVM bytecode structure:");
            println!("// Dispatcher code (78 bytes)");
            let mut byte_offset = 0;
            println!("{:3}: PUSH0", byte_offset); byte_offset += 1;
            println!("{:3}: CALLDATALOAD", byte_offset); byte_offset += 1;
            println!("{:3}: PUSH32 0x{} // magic number q", byte_offset, q.iter().rev().map(|b| format!("{:02x}", b)).collect::<String>()); byte_offset += 33;
            println!("{:3}: MUL", byte_offset); byte_offset += 1;
            println!("{:3}: PUSH32 0x{:08x} // shift amount", byte_offset, shift); byte_offset += 33;
            println!("{:3}: SHR", byte_offset); byte_offset += 1;
            println!("{:3}: PUSH1 0xFF", byte_offset); byte_offset += 2;
            println!("{:3}: AND", byte_offset); byte_offset += 1;
            println!("{:3}: PUSH1 0x06", byte_offset); byte_offset += 2;
            println!("{:3}: MUL", byte_offset); byte_offset += 1;
            println!("{:3}: JUMPDEST", byte_offset); byte_offset += 1;
            println!("{:3}: JUMP", byte_offset); byte_offset += 1;
            
            println!("\n// Function dispatchers (starts at byte 78)");
            println!("// Each function entry point consists of:");
            println!("// JUMPDEST (1 byte)");
            println!("// PUSH3 <function_address> (4 bytes)");
            println!("// JUMP (1 byte)");
            println!("// Total: 6 bytes per function");
            
            let mut selector_to_index = HashMap::new();
            let mut index_to_address = HashMap::new();
            let mut index_to_selector = HashMap::new();
            
            for (&x, &addr) in function_selectors.iter().zip(function_addresses.iter()) {
                let x256 = u32_to_256(x);
                let prod = mul_256(&x256, &q);
                let shifted = shr_256(&prod, shift);
                let result_byte = shifted[0];
                selector_to_index.insert(x, result_byte);
                index_to_address.insert(result_byte, addr);
                index_to_selector.insert(result_byte, x);
            }
            
            // Place function dispatchers at their calculated offsets
            let mut dispatcher_offsets: Vec<(usize, u8, u32, u32)> = Vec::new();
            for i in 0..=255 {
                if let Some(&addr) = index_to_address.get(&(i as u8)) {
                    let selector = index_to_selector.get(&(i as u8)).unwrap();
                    let result_byte = i as u8;
                    let offset = 78 + (result_byte as usize * 6);
                    dispatcher_offsets.push((offset, result_byte, *selector, addr));
                }
            }
            
            // Sort by offset to show them in order
            dispatcher_offsets.sort_by_key(|&(offset, _, _, _)| offset);
            
            // Print dispatchers at their correct offsets
            let mut current_offset = 78;
            for (offset, result_byte, selector, addr) in dispatcher_offsets {
                // Skip printing NOPs, just show the gap in offset
                if current_offset < offset {
                    println!("// Gap from offset {} to {}", current_offset, offset);
                }
                
                // Print the dispatcher
                println!("{:3}: JUMPDEST", offset);
                println!("{:3}: PUSH3 0x{:06x} // Function at 0x{:08x} (selector: 0x{:08x}, result byte: 0x{:02x})", 
                        offset + 1, addr & 0xffffff, addr, selector, result_byte);
                println!("{:3}: JUMP", offset + 5);
                current_offset = offset + 6;
            }
            
            println!("\n// Function code blocks");
            for (i, (&selector, &addr)) in function_selectors.iter().zip(function_addresses.iter()).enumerate() {
                println!("\n// Function at 0x{:08x}", addr);
                println!("0x{:08x}: JUMPDEST", addr);
                println!("// Function {} implementation", i + 1);
                println!("// Selector: 0x{:08x}", selector);
                println!("// ... function code ...");
                println!("0x{:08x}: STOP", addr + 1);
            }
        }
        None => println!("Could not find magic numbers within max attempts"),
    }
}
