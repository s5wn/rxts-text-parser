use clap::{ Parser };
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    // Input path (no glob)
    #[arg(short = 'i', long)]
    path_in: std::path::PathBuf,
    // Output path (no file ext)
    #[arg(short = 'o', long)]
    path_out: std::path::PathBuf,
    // Output file extension
    #[arg(short = 'x', long, default_value_t = String::from("json"))]
    output_ext: String,
}
