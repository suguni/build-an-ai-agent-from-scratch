pub mod http_client;
pub use http_client::HttpClient;

pub async fn get_embeddings(texts: &[&str]) -> anyhow::Result<Vec<Vec<f32>>> {
    let response = HttpClient::new().embedding(texts).await?;
    println!("model: {}, load_dur: {}, total_dur: {}, eval_count: {}",
             response.model, response.load_duration, response.total_duration, response.prompt_eval_count);
    Ok(response.embeddings)
}
