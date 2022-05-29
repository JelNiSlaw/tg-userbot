use std::env;

use serde::{Deserialize, Serialize};

const BASE_URL: &str = "https://api-inference.huggingface.co/models";

#[derive(Serialize)]
struct Request {
    inputs: String,
}

#[derive(Deserialize)]
struct Result {
    generated_text: String,
}

async fn request<S: AsRef<str>>(
    client: reqwest::Client,
    model: S,
    body: Request,
) -> reqwest::Result<Vec<Result>> {
    client
        .post(format!("{BASE_URL}/{}", model.as_ref()))
        .bearer_auth(env::var("HUGGINGFACE_TOKEN").unwrap())
        .json(&body)
        .send()
        .await?
        .json::<Vec<Result>>()
        .await
}

pub async fn gpt_j<S: Into<String>>(client: reqwest::Client, input: S) -> reqwest::Result<String> {
    let mut results = request(
        client,
        "EleutherAI/gpt-j-6B",
        Request {
            inputs: input.into(),
        },
    )
    .await?;

    Ok(results.swap_remove(0).generated_text)
}
