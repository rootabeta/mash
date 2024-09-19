use anyhow::Result;
use clap::Parser;
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

    /// Command and arguments, with %TARGET% in place of the IP
    #[arg(num_args(1..))]
    command: Vec<String>
}

struct Job { 
    // Command, ready to execute
    command: Vec<String>,
    // File to write output to
    stdout_file: Option<String>,

}

fn load_jobs(input_file: String, command: Vec<String>, output_dir: String, no_stdout: bool) -> Result<Vec<Job>> { 
/*    for i in args.command.iter() { 
        println!("{}", i);
    }
*/
    todo!();
}

// Run command, mirroring stdout to %TARGET-argv[0].stdout
// Stdout and stderr are echoed to screen
fn launch_command() { 
    todo!();
}

fn main() {
    let args = Args::parse();

    let num_threads = args.threads.unwrap_or(num_cpus::get());
    let output_dir = args.output.unwrap_or(".".to_string());

    let (mut tx, rx) = spmc::channel::<Job>();

    // Populate queue with jobs
    let jobs = load_jobs(args.input_file, args.command, output_dir, args.no_record).unwrap();

    // Launch threads to consume jobs
    for _ in 0..num_threads { 
        thread::spawn(|| { 
            launch_command(

            );
        });
    }
}
