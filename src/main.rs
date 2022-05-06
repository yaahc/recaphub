use clap::Parser;
use color_eyre::eyre;
use octocrab::models::issues::Issue;

/// Summarize the recent activity of the given user within the given timeframe
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    #[clap(subcommand)]
    command: commands::Commands,

    /// Github personal access token
    #[clap(short, long, env = "GITHUB_TOKEN")]
    github_token: String,
}

mod commands {
    mod reviewers;
    mod user;
    use color_eyre::eyre;
    use reviewers::Reviewers;
    use user::User;

    use clap::Subcommand;
    #[derive(Subcommand, Debug)]
    pub enum Commands {
        /// Lookup activity by user and timeframe
        User(User),
        /// Lookup review activity by repo, timeframe, and tags
        Reviewers(Reviewers),
    }

    impl Commands {
        pub async fn run(&self) -> eyre::Result<()> {
            match self {
                Commands::User(user) => user.run().await,
                Commands::Reviewers(reviewers) => reviewers.run().await,
            }
        }
    }
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    // not an error if dotenv isn't present, so completely discard the results
    let _ = dotenv::dotenv();
    color_eyre::config::HookBuilder::default().install()?;
    let args = Args::parse();
    octocrab::initialise(
        octocrab::Octocrab::builder().personal_token(args.github_token.to_string()),
    )?;

    args.command.run().await?;
    Ok(())
}

trait IssueExt {
    fn repo(&self) -> &str;
    fn owner(&self) -> &str;
}

impl IssueExt for Issue {
    fn repo(&self) -> &str {
        self.repository_url
            .path_segments()
            .expect("repo urls will always have some path segments")
            .rev()
            .next()
            .expect("the last path segment of a repo url is always the repo name")
    }

    fn owner(&self) -> &str {
        self.repository_url
            .path_segments()
            .expect("repo urls will always have some path segments")
            .rev()
            .nth(1)
            .expect("the second to last path segment of a repo url is always the owner name")
    }
}
