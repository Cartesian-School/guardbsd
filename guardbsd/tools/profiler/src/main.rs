// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski

//! GuardBSD Profiling Infrastructure
//! Minimal performance counter and tracing tool

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::time::Instant;

/// Performance counter types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum CounterType {
    Syscall,
    IPC,
    ContextSwitch,
    PageFault,
    TLBFlush,
}

/// Performance sample
#[derive(Debug)]
struct Sample {
    counter: CounterType,
    cycles: u64,
    timestamp: u64,
}

/// Profiler state
struct Profiler {
    samples: Vec<Sample>,
    counters: HashMap<CounterType, u64>,
    start_time: Instant,
}

impl Profiler {
    fn new() -> Self {
        Self {
            samples: Vec::with_capacity(10000),
            counters: HashMap::new(),
            start_time: Instant::now(),
        }
    }

    fn record(&mut self, counter: CounterType, cycles: u64) {
        let timestamp = self.start_time.elapsed().as_nanos() as u64;
        self.samples.push(Sample { counter, cycles, timestamp });
        *self.counters.entry(counter).or_insert(0) += 1;
    }

    fn analyze(&self) -> Report {
        let mut report = Report::default();
        
        for (counter, count) in &self.counters {
            let total_cycles: u64 = self.samples.iter()
                .filter(|s| s.counter == *counter)
                .map(|s| s.cycles)
                .sum();
            
            let avg = if *count > 0 { total_cycles / count } else { 0 };
            
            report.stats.insert(*counter, Stats {
                count: *count,
                total_cycles,
                avg_cycles: avg,
            });
        }
        
        report
    }

    fn export(&self, path: &str) -> std::io::Result<()> {
        let mut file = File::create(path)?;
        writeln!(file, "# GuardBSD Profiling Data")?;
        writeln!(file, "# timestamp,counter,cycles")?;
        
        for sample in &self.samples {
            writeln!(file, "{},{:?},{}", 
                sample.timestamp, sample.counter, sample.cycles)?;
        }
        
        Ok(())
    }
}

/// Performance statistics
#[derive(Debug)]
struct Stats {
    count: u64,
    total_cycles: u64,
    avg_cycles: u64,
}

/// Performance report
#[derive(Debug, Default)]
struct Report {
    stats: HashMap<CounterType, Stats>,
}

impl Report {
    fn print(&self) {
        println!("\n=== GuardBSD Performance Report ===\n");
        println!("{:<20} {:>10} {:>15} {:>12}", 
            "Counter", "Count", "Total Cycles", "Avg Cycles");
        println!("{:-<60}", "");
        
        for (counter, stats) in &self.stats {
            println!("{:<20} {:>10} {:>15} {:>12}", 
                format!("{:?}", counter),
                stats.count,
                stats.total_cycles,
                stats.avg_cycles);
        }
        
        println!("\n");
    }
}

/// Parse trace file from kernel
fn parse_trace(path: &str) -> std::io::Result<Profiler> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut profiler = Profiler::new();
    
    for line in reader.lines() {
        let line = line?;
        if line.starts_with('#') { continue; }
        
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 2 { continue; }
        
        let counter = match parts[0] {
            "syscall" => CounterType::Syscall,
            "ipc" => CounterType::IPC,
            "ctx_switch" => CounterType::ContextSwitch,
            "page_fault" => CounterType::PageFault,
            "tlb_flush" => CounterType::TLBFlush,
            _ => continue,
        };
        
        if let Ok(cycles) = parts[1].parse::<u64>() {
            profiler.record(counter, cycles);
        }
    }
    
    Ok(profiler)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: profiler <trace_file>");
        eprintln!("       profiler --help");
        std::process::exit(1);
    }
    
    if args[1] == "--help" {
        println!("GuardBSD Profiler v1.0.0");
        println!("Minimal performance analysis tool\n");
        println!("Usage: profiler <trace_file>");
        println!("       profiler --export <trace_file> <output.csv>");
        println!("\nTrace format: counter,cycles");
        println!("Counters: syscall, ipc, ctx_switch, page_fault, tlb_flush");
        return;
    }
    
    if args[1] == "--export" && args.len() >= 4 {
        match parse_trace(&args[2]) {
            Ok(profiler) => {
                if let Err(e) = profiler.export(&args[3]) {
                    eprintln!("Export failed: {}", e);
                    std::process::exit(1);
                }
                println!("Exported to {}", args[3]);
            }
            Err(e) => {
                eprintln!("Parse failed: {}", e);
                std::process::exit(1);
            }
        }
        return;
    }
    
    match parse_trace(&args[1]) {
        Ok(profiler) => {
            let report = profiler.analyze();
            report.print();
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
