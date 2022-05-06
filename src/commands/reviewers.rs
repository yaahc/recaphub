use crate::IssueExt;
use chrono::{DateTime, Duration, Utc};
use clap::Args;
use color_eyre::eyre;
use futures::{stream::FuturesOrdered, TryStreamExt};
use octocrab::models::issues::Issue;
use octocrab::models::{issues, pulls};
use std::{
    collections::HashMap,
    future::Future,
    ops::{Add, AddAssign},
};

/// Lookup review activity by repo, timeframe, and labels
#[derive(Args, Debug)]
pub struct Reviewers {
    /// Github repo to query activity
    #[clap(short, long)]
    repo: String,

    /// Timeframe to query against in days
    #[clap(short, long, parse(try_from_str = humantime::parse_duration))]
    timeframe: std::time::Duration,

    /// Github labels to filter by
    #[clap(long)]
    labels: Vec<String>,

    /// Users to filter out of results
    #[clap(short, long)]
    ignored_user: Vec<String>,
}

impl Reviewers {
    pub async fn run(&self) -> eyre::Result<()> {
        let octocrab = octocrab::instance();

        let cutoff = self.cutoff();
        let query = format!(
            "repo:{} sort:created-asc updated:>={} is:pr ",
            self.repo,
            cutoff.date().naive_utc()
        );

        let mut total_stats = PullReviewStats::default();

        for label in &self.labels {
            let query = query.clone() + label;
            dbg!(&query);

            let first_page = octocrab
                .search()
                .issues_and_pull_requests(&query)
                .per_page(25)
                .send()
                .await?;

            let mut current_page = Some(first_page);
            let mut pr_stats = FuturesOrdered::new();

            while let Some(page) = current_page {
                for pr in page.items {
                    let stats = tokio::spawn(self.review_stats(&pr));
                    pr_stats.push(stats);
                }
                current_page = octocrab.get_page(&page.next).await?;
            }

            let stats = pr_stats
                .map_err(eyre::Report::from)
                .try_fold(PullReviewStats::default(), |sum, next| async move {
                    Ok(sum + next?)
                })
                .await?;

            total_stats = total_stats + stats;
        }

        let mut stats = total_stats.0.into_iter().collect::<Vec<_>>();
        stats.sort_by_cached_key(|(_name, stats)| stats.num_comments + stats.num_review_comments);

        println!("# Review Summary:");
        for contributor in stats.into_iter().rev() {
            println!(
                "- {} left {} comments and {} review comments in {} PRs",
                contributor.0,
                contributor.1.num_comments,
                contributor.1.num_review_comments,
                contributor.1.prs_participated_in
            );
        }

        Ok(())
    }

    fn cutoff(&self) -> DateTime<Utc> {
        Utc::now()
            - Duration::from_std(self.timeframe)
                .expect("valid timespans for github queries should fit within a std Duration")
    }

    fn review_comments_within(
        &self,
        issue: &Issue,
    ) -> impl Future<Output = eyre::Result<Vec<pulls::Comment>>> + 'static {
        let pull_num = issue
            .number
            .try_into()
            .expect("issue numbers should be positive");

        let octocrab = octocrab::instance();
        let owner = issue.owner().to_string();
        let repo = issue.repo().to_string();
        let cutoff = self.cutoff();

        async move {
            let first_page = octocrab
                .pulls(owner, repo)
                .list_comments(Some(pull_num))
                .since(cutoff)
                .per_page(100)
                .send()
                .await?;

            let mut reviews = vec![];
            let mut current_page = Some(first_page);
            while let Some(page) = current_page {
                reviews.extend(page.items);
                current_page = octocrab.get_page(&page.next).await?;
            }

            Ok(reviews)
        }
    }

    // I know this is an issue, but that's only because the issues_and_pull_requests endpoint is
    // dumb
    fn comments_within(
        &self,
        issue: &Issue,
    ) -> impl Future<Output = eyre::Result<Vec<issues::Comment>>> + 'static {
        let pull_num = issue
            .number
            .try_into()
            .expect("issue numbers should be positive");

        let octocrab = octocrab::instance();
        let owner = issue.owner().to_string();
        let repo = issue.repo().to_string();
        let cutoff = self.cutoff();

        async move {
            let first_page = octocrab
                .issues(owner, repo)
                .list_comments(pull_num)
                .since(cutoff)
                .per_page(100)
                .send()
                .await?;

            let mut comments = vec![];
            let mut current_page = Some(first_page);
            while let Some(page) = current_page {
                comments.extend(page.items);
                current_page = octocrab.get_page(&page.next).await?;
            }

            Ok(comments)
        }
    }

    fn review_stats(
        &self,
        issue: &Issue,
    ) -> impl Future<Output = eyre::Result<PullReviewStats>> + 'static {
        let comments = tokio::spawn(self.comments_within(issue));
        let reviews = tokio::spawn(self.review_comments_within(issue));
        let ignored_users = self.ignored_user.clone();
        async move {
            let mut stats = PullReviewStats::default();
            let comments = comments.await??;
            for comment in comments {
                let name = comment.user.login;
                if ignored_users.contains(&name) {
                    continue;
                }

                stats
                    .0
                    .entry(name)
                    .or_insert_with(IndividualReviewStats::new)
                    .num_comments += 1;
            }
            let reviews = reviews.await??;
            for review in reviews {
                let name = review.user.login;
                if ignored_users.contains(&name) {
                    continue;
                }
                stats
                    .0
                    .entry(name)
                    .or_insert_with(IndividualReviewStats::new)
                    .num_review_comments += 1;
            }
            Ok(stats)
        }
    }
}

#[derive(Default, Debug)]
struct PullReviewStats(HashMap<String, IndividualReviewStats>);

impl Add for PullReviewStats {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        for (key, value) in rhs.0 {
            *self
                .0
                .entry(key)
                .or_insert_with(IndividualReviewStats::empty) += value
        }
        self
    }
}

#[derive(Debug)]
struct IndividualReviewStats {
    num_comments: usize,
    num_review_comments: usize,
    prs_participated_in: usize,
}

impl IndividualReviewStats {
    fn new() -> Self {
        Self {
            num_comments: 0,
            num_review_comments: 0,
            prs_participated_in: 1,
        }
    }

    fn empty() -> Self {
        Self {
            num_comments: 0,
            num_review_comments: 0,
            prs_participated_in: 0,
        }
    }
}

impl AddAssign for IndividualReviewStats {
    fn add_assign(&mut self, rhs: Self) {
        self.num_comments += rhs.num_comments;
        self.num_review_comments += rhs.num_review_comments;
        self.prs_participated_in += rhs.prs_participated_in;
    }
}
