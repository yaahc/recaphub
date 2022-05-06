use crate::IssueExt;
use chrono::{DateTime, Duration, Utc};
use clap::Args;
use color_eyre::eyre;
use octocrab::models::issues::Issue;
use std::future::Future;
use tokio::task::JoinHandle;

/// Lookup activity by user and timeframe
#[derive(Args, Debug)]
pub struct User {
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

impl User {
    pub async fn run(&self) -> eyre::Result<()> {
        let octocrab = octocrab::instance();
        let cutoff = self.cutoff();
        let query = format!(
            "involves:{} sort:created-asc updated:>={}",
            self.name,
            cutoff.date().naive_utc()
        );

        let first_page = octocrab
            .search()
            .issues_and_pull_requests(&query)
            .per_page(25)
            .send()
            .await?;

        let mut comment_summaries: Vec<JoinHandle<eyre::Result<(Issue, Vec<url::Url>)>>> = vec![];
        let mut current_page = Some(first_page);

        while let Some(page) = current_page {
            for issue in page.items {
                let comments = self.comments_within(&issue);
                let comment_summary_handle = tokio::spawn(async move {
                    let comments = comments.await?;
                    Ok((issue, comments))
                });
                comment_summaries.push(comment_summary_handle);
            }
            current_page = octocrab.get_page(&page.next).await?;
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

    fn cutoff(&self) -> DateTime<Utc> {
        Utc::now()
            - Duration::from_std(self.timeframe)
                .expect("all valid timespans for github queries should fit within a std Duration")
    }

    fn comments_within(
        &self,
        issue: &Issue,
    ) -> impl Future<Output = eyre::Result<Vec<url::Url>>> + 'static {
        let issue_num = issue
            .number
            .try_into()
            .expect("issue numbers are always positive");

        let octocrab = octocrab::instance();
        let owner = issue.owner().to_string();
        let repo = issue.repo().to_string();
        let name = self.name.clone();
        let cutoff = self.cutoff();

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
