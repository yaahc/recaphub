use std::future::Future;

use chrono::{DateTime, Duration, Utc};
use clap::Parser;
use color_eyre::eyre;
use octocrab::{models::issues::Issue, Octocrab, Page};

/// Summarize the recent activity of the given user within the given timeframe
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Github username to query activity
    #[clap(short, long)]
    name: String,

    /// Timeframe to query against in days
    #[clap(short, long, parse(try_from_str = humantime::parse_duration))]
    timeframe: std::time::Duration,

    /// Github personal access token
    #[clap(short, long, env = "GITHUB_TOKEN")]
    github_token: String,
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::config::HookBuilder::default().install()?;

    let mut timeframe = ActivityTimeframe::from_args().await?;

    let mut comment_summaries = vec![];
    while let Some(issue) = timeframe.next_issue().await {
        let issue = issue?;
        let comments = timeframe.comments_within(&issue);
        let comment_summary_handle = tokio::spawn(async move {
            let comments = comments.await?;
            Ok::<_, eyre::Report>((issue, comments))
        });
        comment_summaries.push(comment_summary_handle);
    }

    for handle in comment_summaries {
        let (issue, comments) = handle.await??;
        if comments.is_empty() {
            continue;
        }

        let title = &issue.title;
        let repo = issue.repo();
        let owner = issue.owner();
        println!("- {owner}/{repo}: {title}");
        for comment in comments {
            println!("  - {comment}");
        }
    }

    Ok(())
}

struct ActivityTimeframe {
    octocrab: Octocrab,
    cutoff: DateTime<Utc>,
    name: String,
    current_page: Option<Page<Issue>>,
}

impl ActivityTimeframe {
    async fn from_args() -> eyre::Result<Self> {
        // not an error if dotenv isn't present, so completely discard the results
        let _ = dotenv::dotenv();
        let args = Args::parse();

        let octocrab = octocrab::Octocrab::builder()
            .personal_token(args.github_token)
            .build()?;

        let cutoff = Utc::now() - Duration::from_std(args.timeframe)?;
        let query = format!(
            "involves:{} sort:created-asc updated:>={}",
            args.name,
            cutoff.date().naive_utc()
        );

        let first_page = octocrab
            .search()
            .issues_and_pull_requests(&query)
            .per_page(25)
            .send()
            .await?;

        Ok(Self {
            octocrab,
            cutoff,
            name: args.name,
            current_page: Some(first_page),
        })
    }

    // this shoulda been an async iterator
    async fn next_issue(&mut self) -> Option<eyre::Result<Issue>> {
        let page = self.current_page.as_mut()?;

        if page.items.is_empty() {
            *page = match self.octocrab.get_page(&page.next).await {
                Ok(page) => page?,
                Err(err) => return Some(Err(err.into())),
            };
        }

        let page = self.current_page.as_mut()?;
        page.items.pop().map(Ok)
    }

    fn comments_within(
        &self,
        issue: &Issue,
    ) -> impl Future<Output = eyre::Result<Vec<url::Url>>> + 'static {
        let issue_num = issue
            .number
            .try_into()
            .expect("issue numbers are always positive");

        let octocrab = self.octocrab.clone();
        let cutoff = self.cutoff;
        let owner = issue.owner().to_string();
        let repo = issue.repo().to_string();
        let name = self.name.clone();
        async move {
            let comments = octocrab
                .issues(owner, repo)
                .list_comments(issue_num)
                .since(cutoff)
                .per_page(100)
                .send()
                .await?;

            Ok(comments
                .into_iter()
                .filter(|comment| comment.user.login == name)
                .map(|comment| comment.html_url)
                .collect())
        }
    }
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
            .rev().nth(1)
            .expect("the second to last path segment of a repo url is always the owner name")
    }
}
