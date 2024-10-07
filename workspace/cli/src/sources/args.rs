use std::path::PathBuf;

/// Manage Nix sources
#[derive(Debug, clap::Args)]
#[command(about)]
pub struct Args {
    #[command(subcommand)]
    pub cmd: Cmd,
    /// Path to `sources.json`
    #[arg(long, env, default_value=default_path().into_os_string())]
    pub sources_path: PathBuf,
}

#[derive(Debug, Clone, clap::Subcommand)]
pub enum Cmd {
    /// Add a source
    Add {
        name: String,
        #[command(subcommand)]
        source: Source,
    },
    /// Remove a source
    Remove { name: String },
    /// Update one or all of the sources
    Update { name: Option<String> },
}

#[derive(Debug, Clone, clap::Subcommand)]
#[command(rename_all = "lower")]
pub enum Source {
    Git {
        url: String,
        #[arg(value_name = "ref", long, short = 'b')]
        reference: Option<String>,
        #[arg(value_name = "rev", long, short)]
        revision: Option<String>,
    },
    GitHub {
        owner: String,
        repo: String,
        #[arg(long, short)]
        branch: Option<String>,
        #[arg(long, short, value_name = "rev")]
        revision: Option<String>,
    },
}

impl Args {
    pub fn run(self) {
        use super::*;
        let mut sources = internal::Sources::open(&self.sources_path);
        match self.cmd {
            Cmd::Add { name, source } => {
                let source = match source {
                    Source::Git {
                        url,
                        reference,
                        revision,
                    } => typhon_nix::Source::Git {
                        url,
                        reference,
                        revision,
                    },
                    Source::GitHub {
                        owner,
                        repo,
                        branch,
                        revision,
                    } => typhon_nix::Source::GitHub {
                        owner,
                        repo,
                        branch,
                        revision,
                    },
                };
                sources.add(name, source);
            }
            Cmd::Remove { name } => sources.remove(&name),
            Cmd::Update { name: Some(name) } => sources.update(&name),
            Cmd::Update { name: None } => sources.update_all(),
        }
        sources.write(&self.sources_path);
    }
}

fn default_path() -> PathBuf {
    let mut path = std::env::current_dir().unwrap();
    path.push("sources.json");
    path
}
