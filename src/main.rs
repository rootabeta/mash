use anyhow::Result;
use clap::Parser;
use std::fs::read_to_string;
use std::io::{self, Write};
use std::process::Command;
use std::thread;

/// Multithread anything
#[derive(Parser, Debug)]
struct Args { 
    /// File containing a list of values to insert
    #[arg(short, long)]
    input_file: String,

    /// Directory to capture stdout of each scan to - cwd by default
    #[arg(short, long)]
    output: Option<String>,

    /// Disable recording output to files
    #[arg(short, long, default_value_t = true)]
    no_record: bool,

    /// Number of processes to run concurrently
    #[arg(short, long)]
    threads: Option<usize>,

    /// Command and arguments, with %INPUT% in place of the IP
    #[arg(num_args(1..))]
    command: Vec<String>
}

struct Job { 
    // Command
    command: String,
    // Arguments, preformatted to include targets
    arguments: Vec<String>,
    // File to write output to
    stdout_file: Option<String>,

}

fn read_lines(file: String) -> Result<Vec<String>> { 
    let mut result = Vec::new();

    for line in read_to_string(file)?.lines() { 
        result.push(line.to_string())
    }

    Ok(result)
}

// Run command, mirroring stdout to %TARGET-argv[0].stdout
// Stdout and stderr are echoed to screen
fn launch_command(job: Job) -> Result<String> { 
    let output = Command::new(job.command)
        .args(job.arguments)
        .output()?;

    io::stdout().write_all(&output.stdout)?;
    let _ = io::stdout().flush();

    Ok("Placeholder".to_string())
}

fn main() {
    let args = Args::parse();

    let num_threads = args.threads.unwrap_or(num_cpus::get());
    let output_dir = args.output.unwrap_or(".".to_string());

    let (mut tx, rx) = spmc::channel::<Job>();

    let input_lines = read_lines(args.input_file).expect("Failed to read input file");


    let mut thread_handles = Vec::new();
    // Launch threads to consume jobs
    for _ in 0..num_threads { 
        thread_handles.push(
            thread::spawn(|| { 
                let _ = launch_command(
                    Job { 
                        command: "/bin/ls".to_string(),
                        arguments: vec!["-l".to_string(), "-a".to_string(), "/home".to_string()],
                        stdout_file: Some("test".to_string())
                    }

                );
            })
        );
    }

    // Await completion of all jobs
    for handle in thread_handles { 
        let _res = handle.join();
    }
}
