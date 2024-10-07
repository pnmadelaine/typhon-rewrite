mod runtime;
pub mod sources;

/// Typhon CLI
#[derive(Debug, clap::Parser)]
#[command(disable_help_subcommand = true)]
pub enum Cmd {
    Sources(sources::Args),
}

impl Cmd {
    pub fn run(self) {
        match self {
            Self::Sources(args) => args.run(),
        }
    }
}
