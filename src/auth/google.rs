use anyhow::{bail, Context, Result};
use colored::*;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use url::Url;

const CLIENT_ID: &str = "407408718192.apps.googleusercontent.com"; // Standard public client ID for desktop apps (placeholder, recommend replacing)
const REDIRECT_URI: &str = "http://127.0.0.1:8080/callback";
const AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleToken {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: u64,
    pub token_type: String,
    pub expires_at_timestamp: Option<u64>,
}

pub struct GoogleAuth;

impl GoogleAuth {
    fn token_file_path() -> Option<PathBuf> {
        dirs::home_dir().map(|h| h.join(".pi").join("google_oauth.json"))
    }

    pub fn save_token(token: &GoogleToken) -> Result<()> {
        if let Some(path) = Self::token_file_path() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            let json = serde_json::to_string_pretty(token)?;
            fs::write(&path, json)?;
        }
        Ok(())
    }

    pub fn load_token() -> Result<Option<GoogleToken>> {
        if let Some(path) = Self::token_file_path() {
            if path.exists() {
                let content = fs::read_to_string(&path)?;
                if let Ok(token) = serde_json::from_str::<GoogleToken>(&content) {
                    return Ok(Some(token));
                }
            }
        }
        Ok(None)
    }

    pub async fn refresh_token_if_needed(token: &mut GoogleToken) -> Result<bool> {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_secs();
        let expires_at = token.expires_at_timestamp.unwrap_or(0);

        let client_id = env::var("GOOGLE_CLIENT_ID").unwrap_or_else(|_| CLIENT_ID.to_string());
        let client_secret = env::var("GOOGLE_CLIENT_SECRET").unwrap_or_else(|_| "".to_string());

        // If expires in less than 5 minutes, refresh
        if now + 300 >= expires_at {
            if let Some(refresh) = &token.refresh_token {
                let client = Client::new();
                let params = [
                    ("client_id", client_id.as_str()),
                    ("client_secret", client_secret.as_str()),
                    ("refresh_token", refresh.as_str()),
                    ("grant_type", "refresh_token"),
                ];

                let resp = client.post(TOKEN_URL).form(&params).send().await?;
                if resp.status().is_success() {
                    let mut new_token: GoogleToken = resp.json().await?;
                    // Keep old refresh token if not returned
                    if new_token.refresh_token.is_none() {
                        new_token.refresh_token = Some(refresh.clone());
                    }
                    new_token.expires_at_timestamp = Some(now + new_token.expires_in);
                    *token = new_token.clone();
                    Self::save_token(&new_token)?;
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    pub async fn authenticate_via_browser() -> Result<GoogleToken> {
        let client_id = env::var("GOOGLE_CLIENT_ID").unwrap_or_default();
        let client_secret = env::var("GOOGLE_CLIENT_SECRET").unwrap_or_default();

        if client_id.is_empty() || client_id == CLIENT_ID {
            println!("\n{}", "🛑 Google OAuth Setup Required".bright_red().bold());
            println!("To use browser-based login, you must provide your own Google Cloud OAuth credentials.");
            println!("Google's security policies forbid us from hardcoding a shared public Client ID for AI scopes.\n");
            
            println!("{}", "How to fix this in 3 minutes:".bright_yellow().bold());
            println!("1. Go to {}", "https://console.cloud.google.com/".bright_cyan().underline());
            println!("2. Create a new Project, then go to APIs & Services -> OAuth consent screen.");
            println!("3. Go to Credentials -> Create Credentials -> OAuth client ID.");
            println!("4. Choose 'Desktop app' (or 'Web application' with redirect URI: http://127.0.0.1:8080/callback).");
            println!("5. Export the credentials in your terminal:\n");
            
            println!("   {}", "export GOOGLE_CLIENT_ID=\"your-client-id.apps.googleusercontent.com\"".bright_green());
            println!("   {}", "export GOOGLE_CLIENT_SECRET=\"your-client-secret\"".bright_green());
            
            println!("\nOr, if you don't want to do this, just use a free API key instead!");
            println!("Get one at {} and run: {}", "https://aistudio.google.com/".bright_cyan().underline(), "export GEMINI_API_KEY=\"key\"".bright_green());
            
            bail!("Missing GOOGLE_CLIENT_ID and GOOGLE_CLIENT_SECRET.");
        }

        let state = uuid::Uuid::new_v4().to_string();
        let auth_url = format!(
            "{}?client_id={}&redirect_uri={}&response_type=code&scope=https://www.googleapis.com/auth/generative-language.retriever%20https://www.googleapis.com/auth/cloud-platform&state={}&access_type=offline&prompt=consent",
            AUTH_URL, client_id, REDIRECT_URI, state
        );

        println!("Opening browser to authenticate with Google Gemini...");
        println!("If the browser does not open automatically, click this link:\n\n{}\n", auth_url);
        
        if webbrowser::open(&auth_url).is_err() {
            println!("Could not open browser automatically. Please copy and paste the URL above.");
        }

        // Start local TCP server
        let listener = TcpListener::bind("127.0.0.1:8080")
            .with_context(|| "Failed to bind to 127.0.0.1:8080. Is the port already in use?")?;
        
        println!("Waiting for authorization code on localhost:8080...");

        let mut auth_code = String::new();
        let mut returned_state = String::new();

        for stream in listener.incoming() {
            let mut stream = stream?;
            let mut reader = BufReader::new(&stream);
            let mut request_line = String::new();
            reader.read_line(&mut request_line)?;

            if request_line.starts_with("GET") {
                let parts: Vec<&str> = request_line.split_whitespace().collect();
                if parts.len() > 1 {
                    let uri = format!("http://localhost{}", parts[1]);
                    if let Ok(parsed_url) = Url::parse(&uri) {
                        for (k, v) in parsed_url.query_pairs() {
                            if k == "code" {
                                auth_code = v.to_string();
                            } else if k == "state" {
                                returned_state = v.to_string();
                            }
                        }
                    }
                }

                // Send success HTML
                let response = "HTTP/1.1 200 OK\r\n\r\n<html><body><h1>Authentication Successful!</h1><p>You can close this window and return to Pi Agent.</p></body></html>";
                stream.write_all(response.as_bytes())?;
                break;
            }
        }

        if auth_code.is_empty() {
            bail!("Failed to extract authorization code from browser callback.");
        }

        if returned_state != state {
            bail!("State mismatch! Potential CSRF attack.");
        }

        println!("Authorization code received. Exchanging for access token...");

        // Exchange code for token
        let client = Client::new();
        let params = [
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("code", &auth_code),
            ("redirect_uri", REDIRECT_URI),
            ("grant_type", "authorization_code"),
        ];

        let resp = client.post(TOKEN_URL).form(&params).send().await?;
        if !resp.status().is_success() {
            let err = resp.text().await?;
            bail!("Failed to exchange token. Google API returned: {}", err);
        }

        let mut token: GoogleToken = resp.json().await?;
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_secs();
        token.expires_at_timestamp = Some(now + token.expires_in);

        Self::save_token(&token)?;
        println!("✅ Successfully authenticated and saved OAuth credentials.");
        
        Ok(token)
    }
}