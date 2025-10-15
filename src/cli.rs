use clap::{ Parser };
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    // Input path (no glob)
    #[arg(short = 'c', long = "as_const", default_value_t = false)]
    pub add_const: bool,

    pub path_in: std::path::PathBuf,
    // Output path (no file ext)
    pub path_out: std::path::PathBuf,
    // Output file extension
    #[arg(short = 'x', long = "output_ext", default_value = "json")]
    pub output_ext: String,
}
