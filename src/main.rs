use anyhow::{bail, Result};
use clap::Parser;
use std::fs::{File, read_to_string};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
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
    #[arg(short, long, default_value_t = false)]
    no_record: bool,

    /// Number of processes to run concurrently
    #[arg(short, long)]
    threads: Option<usize>,

    /// Allow clobbering of filenames
    #[arg(short, long, default_value_t = false)]
    clobber: bool,

    /// Custom prefix for stdout files
    #[arg(short, long)]
    prefix: Option<String>,

    /// Command and arguments, with %INPUT% in place of the IP
    #[arg(num_args(1..))]
    command: Vec<String>
}

struct Job { 
    // Command
    command: String,
    // Arguments, preformatted to include targets
    arguments: Vec<String>,
    // Filepath to write output to
    stdout_file: PathBuf,
    // Allow clobbering files
    clobber: bool,

    // Use stdout or not
    use_stdout: bool
}

// Read a file line by line
fn read_lines(file: String) -> Result<Vec<String>> { 
    let mut result = Vec::new();

    for line in read_to_string(file)?.lines() { 
        result.push(line.to_string())
    }

    Ok(result)
}

// Run command, mirroring stdout to %TARGET-argv[0].stdout
// Stdout and stderr are echoed to screen
fn launch_command(job: Job) -> Result<()> { 
    // If the output file already exists and we do not wish to clobber, 
    // then abort before running any command
    let filepath = Path::new(&job.stdout_file);
    if job.use_stdout && !job.clobber {
        if filepath.exists() { 
            // Skip running command we've already run before
            bail!("Refusing to clobber file");
        }
    }

    let output = Command::new(job.command)
        .args(job.arguments)
        .output()?;

    io::stdout().write_all(&output.stdout)?;
    io::stderr().write_all(&output.stderr)?;
    if job.use_stdout { 
        let _ = File::create(filepath)?.write_all(&output.stderr);
    }
    let _ = io::stdout().flush();

    Ok(())
}

// This is the worker thread 
fn job_worker(rx: spmc::Receiver<Job>) { 
    while let Ok(job) = rx.try_recv() { 
        let _ = launch_command(
            job
        );
    }
}

// Loop over all args, replacing %INPUT% with the line
fn process_args(arguments: &Vec<String>, input: &String) -> Vec<String> { 
    let mut result = Vec::new();
    for argument in arguments.iter().skip(1) { 
        result.push(
            argument.replace("%INPUT%", input.trim())
        )
    }
    result
}

// Take command like nmap -sV 1.2.3.4 and return ./outdir/nmap_-sV_1.2.3.4.stdout
fn generate_stdout_file(directory: &String, prefix: &Option<String>, command: &String, input: &String) -> PathBuf { 
    let stripped_input = input.trim().replace(" ", "_");
    let filename = {
        if let Some(prefixstr) = prefix { 
            format!("{}#{}_{}.stdout", prefixstr, command, stripped_input)
        } else { 
            format!("{}_{}.stdout", command, stripped_input)
        }
    };

    let filepath = Path::new(directory).join(filename);

    filepath
}

// Create a file to store stdout results in, in the specified directory

fn main() {
    let args = Args::parse();

    let num_threads = args.threads.unwrap_or(num_cpus::get());
    let output_dir = args.output.unwrap_or(".".to_string());
    let allow_clobber = args.clobber;

    let (mut tx, rx) = spmc::channel::<Job>();

    let input_lines = read_lines(args.input_file).expect("Failed to read input file");

    for line in input_lines { 
        let job = Job { 
            command: args.command[0].clone(),
            arguments: process_args(&args.command, &line),
            clobber: allow_clobber.clone(),
            use_stdout: !args.no_record,
            stdout_file: generate_stdout_file(&output_dir, &args.prefix, &args.command[0], &line)
        };
        let _ = tx.send(
            job
        );
    }

    let mut thread_handles = Vec::new();
    // Launch threads to consume jobs
    for _ in 0..num_threads { 
        let crx = rx.clone();
        thread_handles.push(
            thread::spawn(move|| { 
                job_worker(crx);
            })
        );
    }

    // While there remains any thread that is not yet finished, wait
    while thread_handles.iter().any(|handle| {!handle.is_finished()}) { 
        // TODO: Progress bar that counts jobs completed and gives % and ETA
    }
}
