use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Source {
    #[serde(rename = "git")]
    Git {
        url: String,
        #[serde(rename = "ref")]
        reference: Option<String>,
        #[serde(rename = "rev")]
        revision: Option<String>,
    },
    #[serde(rename = "codeberg")]
    Codeberg {
        #[serde(default = "default_codeberg_instance")]
        #[serde(skip_serializing_if = "is_default_codeberg_instance")]
        instance: String,
        owner: String,
        repo: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        branch: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none", rename = "rev")]
        revision: Option<String>,
    },
    #[serde(rename = "github")]
    GitHub {
        owner: String,
        repo: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        branch: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none", rename = "rev")]
        revision: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Lock {
    #[serde(rename = "git")]
    Git {
        url: String,
        #[serde(rename = "ref")]
        reference: String,
        #[serde(rename = "rev")]
        revision: String,
    },
    #[serde(rename = "github")]
    GitHub {
        owner: String,
        repo: String,
        #[serde(rename = "rev")]
        revision: String,
        sha256: String,
    },
}

impl Lock {
    pub fn expr(&self) -> String {
        match self {
            Self::Git {
                url,
                reference,
                revision,
            } => {
                format!("(builtins.fetchGit {{ url = \"{url}\"; ref = \"{reference}\"; rev = \"{revision}\"; }})")
            }
            Self::GitHub {
                owner,
                repo,
                revision,
                sha256,
            } => {
                let url = format!("https://github.com/{owner}/{repo}/archive/{revision}.tar.gz");
                format!("(builtins.fetchTarball {{ url = \"{url}\"; sha256 = \"{sha256}\"; }})")
            }
        }
    }
}

fn default_codeberg_instance() -> String {
    "codeberg.org".to_owned()
}

fn is_default_codeberg_instance(instance: &str) -> bool {
    instance == default_codeberg_instance()
}
