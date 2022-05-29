use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct Payload {
    context: String,
    top_p: f32,
    temp: f32,
    response_length: i16,
    remove_input: bool,
}

#[derive(Deserialize)]
struct Response {
    generated_text: String,
}

pub async fn gpt_j<S: Into<String>>(client: reqwest::Client, input: S) -> reqwest::Result<String> {
    let response = client
        .post("https://api.eleuther.ai/completion")
        .json(&Payload {
            context: input.into(),
            top_p: 0.9,
            temp: 0.75,
            response_length: 64,
            remove_input: false,
        })
        .send()
        .await?;

    Ok(response
        .error_for_status()?
        .json::<Vec<Response>>()
        .await?
        .swap_remove(0)
        .generated_text)
}
