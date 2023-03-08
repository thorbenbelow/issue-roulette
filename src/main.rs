use clap::Parser;
use rand::seq::SliceRandom;

/// Simple program to choose a random open issue to work on
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of username or organization
    #[arg(short, long)]
    username: String,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct Repo {
    full_name: String,
    private: bool,
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

use reqwest::header::{HeaderMap, HeaderValue};
#[tokio::main]
async fn main() {
    let args = Args::parse();
    let client = build_http_client().expect("Failed to build http client");
    let repos = client
        .get(format!(
            "https://api.github.com/users/{}/repos",
            args.username
        ))
        .send()
        .await
        .expect("Failed to retrieve repositories")
        .json::<Vec<Repo>>()
        .await
        .expect("Failed to parse json");

    let filtered_repos = repos
        .iter()
        .filter(|repo| !repo.fork && repo.has_issues && repo.open_issues > 0)
        .collect::<Vec<_>>();
    let repo = filtered_repos
        .choose(&mut rand::thread_rng())
        .expect("No viable repos to choose issues from");

    let issues = client
        .get(format!(
            "https://api.github.com/repos/{}/issues",
            repo.full_name
        ))
        .send()
        .await
        .expect(&format!("Failed to retrieve issues for {}", repo.full_name))
        .json::<Vec<Issue>>()
        .await
        .expect(&format!("Failed to parse issues for {}", repo.full_name));

    let issue = issues.choose(&mut rand::thread_rng()).expect("No viable issue found.");
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
        .user_agent("task-roulette")
        .default_headers(headers)
        .build()
}
