use clap::Parser;
use rand::seq::SliceRandom;
use reqwest::{
    header::{HeaderMap, HeaderValue},
    StatusCode,
};

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

    /// Authorization token to include private repositories. Can also be supplied via ENV: ISSUE_ROULETTE_TOKEN
    #[arg(short, long)]
    token: Option<String>,
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
    html_url: String,
}

impl std::fmt::Display for Issue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {} -> {}", self.number, self.title, self.html_url)
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let token = get_token(args.token).expect("Failed to build Auth token header.");
    let client = build_http_client(&token).expect("Failed to build http client.");

    let repos_req = match token {
        Some(_) => get_all_repos(&client).await,
        None => get_public_repos(&client, args.username).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error>),
    };
    let repos = repos_req.expect("Failed to retrieve repositories.");

    println!("Choosing issue from {} repositories...", repos.len());
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

fn build_http_client(token: &Option<HeaderValue>) -> Result<reqwest::Client, reqwest::Error> {
    let mut headers = HeaderMap::new();
    headers.insert(
        "Accept",
        HeaderValue::from_static("application/vnd.github+json"),
    );
    headers.insert(
        "X-Github-Api-Version",
        HeaderValue::from_static("2022-11-28"),
    );

    if let Some(token) = token {
        headers.insert(reqwest::header::AUTHORIZATION, token.clone());
    }

    reqwest::Client::builder()
        .user_agent("issue-roulette")
        .default_headers(headers)
        .build()
}

#[derive(Debug, Clone)]
struct BadRequestError(u16, String);
impl std::fmt::Display for BadRequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]: {}", self.0, self.1)
    }
}
impl std::error::Error for BadRequestError {}

async fn get_all_repos(client: &reqwest::Client) -> Result<Vec<Repo>, Box<dyn std::error::Error>> {
    let res = client
        .get("https://api.github.com/user/repos?per_page=100")
        .send()
        .await?;

    let status = res.status();
    if status != StatusCode::OK {
        let text = res.text().await?;
        return Err(Box::new(BadRequestError(status.as_u16(), text)));
    }

    let json = res.json::<Vec<Repo>>().await?;
    Ok(json)
}

async fn get_public_repos(
    client: &reqwest::Client,
    username: String,
) -> Result<Vec<Repo>, reqwest::Error> {
    client
        .get(format!(
            "https://api.github.com/users/{}/repos?per_page=100",
            username
        ))
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

fn get_token(
    token: Option<String>,
) -> Result<Option<HeaderValue>, reqwest::header::InvalidHeaderValue> {
    if let Some(token) = token.or(std::env::var("ISSUE_ROULETTE_TOKEN").ok()) {
        let value = HeaderValue::from_str(&format!("Bearer {}", token))?;
        Ok(Some(value))
    } else {
        Ok(None)
    }
}
