use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ExecutionEmbedding {
    pub id: String,
    pub repository_id: String,
    pub commit_sha: String,
    pub fingerprint_hash: String,
    pub embedding: Vec<f32>,
    pub language: String,
    pub framework: String,
    pub runtime: String,
    pub created_at: u64,
}

pub fn fingerprint_embedding(fingerprint_hash: &str) -> Vec<f32> {
    let mut values = vec![0.0; 8];
    let len = values.len();
    for (idx, byte) in fingerprint_hash.as_bytes().iter().enumerate() {
        values[idx % len] += f32::from(*byte) / 255.0;
    }
    values
}

#[cfg(test)]
mod tests {
    use super::fingerprint_embedding;

    #[test]
    fn fingerprint_embedding_is_deterministic() {
        let first = fingerprint_embedding("abc123");
        let second = fingerprint_embedding("abc123");
        assert_eq!(first, second);
        assert_eq!(first.len(), 8);
    }
}
