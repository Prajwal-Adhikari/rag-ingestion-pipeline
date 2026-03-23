use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use tonic::transport::Channel;

pub mod splitter {
    tonic::include_proto!("splitter");
}

use splitter::splitter_client::SplitterClient;
use splitter::SplitRequest;

pub struct SemanticChunker {
    client: SplitterClient<Channel>,
    model: TextEmbedding,
    max_words: usize,
    percentile: f32,
}

impl SemanticChunker {
    pub async fn new(
        max_words: usize,
        percentile: f32,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        log::info!("Connecting to gRPC sidecar...");

        let client = SplitterClient::connect("http://[::1]:50051").await?;
        log::info!("Loading embedding model...");
        let model = TextEmbedding::try_new(InitOptions::new(EmbeddingModel::BGESmallENV15))?;
        log::info!("Embedding model ready.");
        Ok(Self {
            client,
            model,
            max_words,
            percentile,
        })
    }

    pub async fn chunk(&mut self, text: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let response = self
            .client
            .split(SplitRequest {
                text: text.to_string(),
            })
            .await?;
        let sentences = response.into_inner().sentences;
        log::info!("Got {} sentences from sidecar", sentences.len());
        if sentences.len() <= 1 {
            return Ok(sentences);
        }

        //embed all sentences
        let embeddings = self.model.embed(sentences.clone(), None)?;

        //cosine similarity between adjacent sentences
        let similarities: Vec<f32> = embeddings
            .windows(2)
            .map(|w| cosine_similarity(&w[0], &w[1]))
            .collect();

        //find boundaries using percentile threshold
        let threshold = percentile_threshold(&similarities, self.percentile);
        log::info!("Similarity threshold: {:.3}", threshold);

        let boundaries: Vec<usize> = similarities
            .iter()
            .enumerate()
            .filter(|(_, &sim)| sim < threshold)
            .map(|(i, _)| i)
            .collect();
        log::info!("Found {} boundaries", boundaries.len());

        //group sentences into chunks
        let raw_chunks = group_by_boundaries(&sentences, &boundaries);

        //enforce max size cap
        let chunks = enforce_max_size(raw_chunks, self.max_words);
        log::info!("Produced {} chunks", chunks.len());

        Ok(chunks)
    }
}

/*
Formula:
cosine similarity= A⋅B​ / ∥A∥⋅∥B∥
𝐴⋅𝐵 → dot product
∥𝐴∥,∥B∥ → magnitudes (Euclidean norms)

Interpretation:
1.0 → identical direction
0.0 → orthogonal (no similarity)
-1.0 → opposite direction
*/

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    dot / { norm_a * norm_b }
}

fn percentile_threshold(similarities: &[f32], percentile: f32) -> f32 {
    let mut sorted = similarities.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let index = (percentile * sorted.len() as f32) as usize;
    sorted[index.min(sorted.len() - 1)]
}

fn group_by_boundaries(sentences: &[String], boundaries: &[usize]) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut start = 0;
    for &boundary in boundaries {
        let chunk = sentences[start..=boundary].join(" ");
        if !chunk.trim().is_empty() {
            chunks.push(chunk);
        }
        start = boundary + 1;
    }
    if start < sentences.len() {
        let chunk = sentences[start..].join(" ");
        if !chunk.trim().is_empty() {
            chunks.push(chunk);
        }
    }
    chunks
}

fn enforce_max_size(chunks: Vec<String>, max_words: usize) -> Vec<String> {
    let mut result = Vec::new();
    for chunk in chunks {
        let words: Vec<&str> = chunk.split_whitespace().collect();
        if words.len() <= max_words {
            result.push(chunk);
        } else {
            for window in words.chunks(max_words) {
                result.push(window.join(" "));
            }
        }
    }
    result
}
