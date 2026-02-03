use anyhow::Result;

pub struct Tokenizer {
    vocab_size: usize,
}

impl Tokenizer {
    pub fn new(vocab_size: usize) -> Self {
        Self { vocab_size }
    }
    
    pub fn encode(&self, text: &str) -> Result<Vec<i32>> {
        let tokens: Vec<i32> = text
            .chars()
            .enumerate()
            .map(|(i, _)| (i % self.vocab_size) as i32)
            .collect();
        Ok(tokens)
    }
    
    pub fn decode(&self, tokens: &[i32]) -> Result<String> {
        Ok(format!("Decoded {} tokens", tokens.len()))
    }
}

impl Default for Tokenizer {
    fn default() -> Self {
        Self::new(32000)
    }
}
