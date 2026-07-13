use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct HealthResponse {
  // pub status: String,
  pub anonymous_token: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LoginRequest {
  pub token: String,
}

#[derive(Debug, Deserialize)]
pub struct SessionResponse {
  pub session_uuid: String,
  pub user_uuid: String,
  // pub expires_at: String,
}

pub async fn perform_anonymous_login(server_url: String) -> Result<SessionResponse, String> {
  let client = reqwest::Client::new();
  let base = server_url.trim_end_matches('/').to_string();

  println!("[INFO] LOGIN | GET health check → {}", base);

  let health: HealthResponse = client
    .get(format!("{}/", base))
    .send()
    .await
    .map_err(|e| format!("Health check failed: {}", e))?
    .json()
    .await
    .map_err(|e| format!("Parse health failed: {}", e))?;

  let token = health.anonymous_token.ok_or("Anonymous login not enabled on server")?;
  println!("[INFO] LOGIN | Anonymous token received: {}", token);

  println!("[INFO] LOGIN | POST login with token → {}", base);

  let session: SessionResponse = client
    .post(format!("{}/", base))
    .json(&LoginRequest {
      token,
    })
    .send()
    .await
    .map_err(|e| format!("Login POST failed: {}", e))?
    .json()
    .await
    .map_err(|e| format!("Parse session failed: {}", e))?;

  println!("[INFO] LOGIN | SUCCESS! Session UUID: {}", session.session_uuid);
  println!("[INFO] LOGIN | User UUID: {}", session.user_uuid);

  Ok(session)
}
