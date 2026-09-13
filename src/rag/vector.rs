use crate::ollama::get_embeddings;

pub fn fixed_length_chunking(
    text: &str,
    chunk_size: usize,
    overlap: usize
) -> Vec<String> {
    let text = text.as_bytes();
    let mut chunks: Vec<String> = vec![];
    let mut start = 0;

    while start < text.len() {
        let end = usize::min(start + chunk_size, text.len());
        let chunk = String::from_utf8_lossy(&text[start..end]);
        chunks.push(chunk.to_string());
        if end == text.len() {
            break;
        }
        start = end - overlap;
    }

    chunks
}

pub async fn vector_search<R: AsRef<[f32]>>(query: &str, chunk_embeddings: &[R], top_k: usize) -> anyhow::Result<Vec<(usize, f32)>> {
    let query_embedding = get_embeddings(&[query]).await?;

    let mut similarities = chunk_embeddings.iter()
        .map(|embedding| innr::cosine(&query_embedding[0], embedding.as_ref()))
        .enumerate()
        .collect::<Vec<_>>();

    similarities
        .sort_unstable_by(|(_, a), (_, b)| b.total_cmp(a));

    Ok(similarities.into_iter().take(top_k).collect::<Vec<_>>())
}

#[cfg(test)]
mod tests {
    use crate::rag::vector::fixed_length_chunking;

    #[test]
    fn test_fixed_length_chunking() {
        let chunk = fixed_length_chunking("abcdef", 3, 1);
        assert_eq!(chunk.len(), 3);
        assert_eq!(chunk[0], "abc");
        assert_eq!(chunk[1], "cde");
        assert_eq!(chunk[2], "ef");
    }
}