use serde::Deserialize;
use std::env;
use std::process::ExitCode;

const API_BASE: &str = "https://www.duolingo.com/2017-06-30";

#[derive(Debug, Deserialize)]
struct UserProfile {
    username: String,
    #[serde(rename = "totalXp")]
    total_xp: u64,
    streak: Option<u64>,
}

fn user_id_from_jwt(jwt: &str) -> Result<u64, String> {
    let payload = jwt
        .split('.')
        .nth(1)
        .ok_or("JWT is missing its payload segment")?;

    use base64::Engine;
    let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(payload.trim_end_matches('='))
        .map_err(|e| format!("failed to base64-decode JWT payload: {e}"))?;

    let claims: serde_json::Value =
        serde_json::from_slice(&decoded).map_err(|e| format!("JWT payload is not JSON: {e}"))?;

    claims["sub"]
        .as_u64()
        .ok_or_else(|| "JWT payload has no numeric 'sub' claim".to_string())
}

fn fetch_profile(client: &reqwest::blocking::Client, jwt: &str, user_id: u64) -> Result<UserProfile, String> {
    let response = client
        .get(format!("{API_BASE}/users/{user_id}"))
        .bearer_auth(jwt)
        .send()
        .map_err(|e| format!("request failed: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("Duolingo API returned HTTP {}", response.status()));
    }

    response
        .json::<UserProfile>()
        .map_err(|e| format!("failed to parse profile response: {e}"))
}

fn run() -> Result<(), String> {
    dotenvy::dotenv().map_err(|e| format!("failed to load .env: {e}"))?;

    let jwt = env::var("DUOLINGO_JWT_TOKEN")
        .map_err(|_| "DUOLINGO_JWT_TOKEN is not set in .env".to_string())?;

    let user_id = user_id_from_jwt(&jwt)?;

    let client = reqwest::blocking::Client::new();
    let profile = fetch_profile(&client, &jwt, user_id)?;

    println!("Logged in as: {}", profile.username);
    println!("Total XP:     {}", profile.total_xp);
    println!("Streak:       {}", profile.streak.unwrap_or(0));

    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
