use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Location of the Paradox DB files
    #[arg(short, long)]
    pub location: String,

    /// Optional port if running SSE (not implemented, stdio by default)
    #[arg(short, long)]
    pub port: Option<u16>,

    /// Permit editing the database
    #[arg(short, long, default_value_t = false)]
    pub permit_editing: bool,
}
