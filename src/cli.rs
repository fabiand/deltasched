use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short, default_value="yaml")]
    pub output: OutputFormat,

    #[command(subcommand)]
    pub command: Commands
}

#[derive(ValueEnum, Debug, Clone)]
pub enum OutputFormat {
    Yaml,
    Human/*,
    MermaidGantt*/
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Output an example schedule skeleton
    Example {},

    /// Create a new schedule
    New {
        /// Name of the project release
        #[arg(long)]
        name: String,
        /// The schedule skeleton to use
        #[arg(long)]
        from_skeleton: String,
        /// The known date, a tuple of <milestone alias>:<yyyy-mm-dd>
        #[arg(long)]
        with_due_date: String
    },

    /// Replan an existing schedule
    /// It is expected that to be replanned dates are null
    Replan {
        #[arg(long)]
        schedule: String
    }
}

