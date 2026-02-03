use crate::inference::GenerationConfig;

pub struct Sampler {
    config: GenerationConfig,
}

impl Sampler {
    pub fn new(config: GenerationConfig) -> Self {
        Self { config }
    }
    
    pub fn sample(&self, logits: &[f32]) -> usize {
        let mut max_idx = 0;
        let mut max_val = logits[0];
        
        for (i, &val) in logits.iter().enumerate() {
            if val > max_val {
                max_val = val;
                max_idx = i;
            }
        }
        
        max_idx
    }
    
    pub fn apply_temperature(&self, logits: &mut [f32]) {
        for logit in logits.iter_mut() {
            *logit /= self.config.temperature;
        }
    }
}
