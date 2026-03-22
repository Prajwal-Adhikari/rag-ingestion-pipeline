use tonic::transport::Channel;

pub mod splitter {
    tonic::include_proto!("splitter");
}

use splitter::splitter_client::SplitterClient;
use splitter::SplitRequest;

pub struct SemanticChunker {
    client: SplitterClient<Channel>,
}

impl SemanticChunker {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let client = SplitterClient::connect("http://[::1]:50051").await?;
        Ok(Self { client })
    }

    pub async fn chunk(&mut self, text: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let response = self
            .client
            .split(SplitRequest {
                text: text.to_string(),
            })
            .await?;
        let sentences = response.into_inner().sentences;
        println!("Got {} sentences from sidecar", sentences.len());
        for (i, s) in sentences.iter().enumerate() {
            println!("[{}] {}", i, s);
        }
        Ok(sentences)
    }
}
