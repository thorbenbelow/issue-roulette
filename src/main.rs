use clap::Parser;
use rand::seq::SliceRandom;
use reqwest::header::{HeaderMap, HeaderValue};

/// Simple program to choose a random open issue to work on.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The handle for the GitHub user account.
    #[arg(short, long)]
    username: String,

    /// Include forked repositories. Defaults to false.
    #[arg(long)]
    include_forked_repos: bool,

    /// Authorization token to include private repositories
    #[arg(short, long)]
    token: Option<String>
}

#[derive(serde::Deserialize, Debug)]
struct Repo {
    full_name: String,
    fork: bool,
    has_issues: bool,
    open_issues: u32,
}

impl std::fmt::Display for Repo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.full_name)
    }
}

#[derive(serde::Deserialize, Debug)]
struct Issue {
    title: String,
    number: u32,
    url: String,
}

impl std::fmt::Display for Issue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {} -> {}", self.number, self.title, self.url)
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let client = build_http_client().expect("Failed to build http client.");

    let repos = get_repos(&client, args.username)
        .await
        .expect("Failed to retrieve repositories.");
    let filtered_repos = repos
        .iter()
        .filter(|repo| repo.has_issues && repo.open_issues > 0)
        .filter(|repo| args.include_forked_repos || !repo.fork)
        .collect::<Vec<_>>();
    let repo = filtered_repos
        .choose(&mut rand::thread_rng())
        .expect("No viable repos to choose issues from.");
    let issues = get_issues(&client, &repo)
        .await
        .expect("Failed to retrieve issues.");
    let issue = issues
        .choose(&mut rand::thread_rng())
        .expect("No viable issue found.");
    println!("🌟🦄 {} 🦄🌟", issue);
}

fn build_http_client() -> Result<reqwest::Client, reqwest::Error> {
    let mut headers = HeaderMap::new();
    headers.insert(
        "Accept",
        HeaderValue::from_static("application/vnd.github+json"),
    );
    headers.insert(
        "X-Github-Api-Version",
        HeaderValue::from_static("2022-11-28"),
    );

    reqwest::Client::builder()
        .user_agent("issue-roulette")
        .default_headers(headers)
        .build()
}

async fn get_repos(
    client: &reqwest::Client,
    username: String,
) -> Result<Vec<Repo>, reqwest::Error> {
    client
        .get(format!("https://api.github.com/users/{}/repos", username))
        .send()
        .await?
        .json::<Vec<Repo>>()
        .await
}

async fn get_issues(client: &reqwest::Client, repo: &Repo) -> Result<Vec<Issue>, reqwest::Error> {
    client
        .get(format!(
            "https://api.github.com/repos/{}/issues",
            repo.full_name
        ))
        .send()
        .await?
        .json::<Vec<Issue>>()
        .await
}
