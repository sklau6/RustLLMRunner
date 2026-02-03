pub mod accelerator;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcceleratorType {
    CPU,
    CUDA,
    Metal,
    ROCm,
}

pub fn detect_accelerator() -> AcceleratorType {
    #[cfg(target_os = "macos")]
    {
        if metal_available() {
            tracing::info!("Metal GPU acceleration available");
            return AcceleratorType::Metal;
        }
    }
    
    #[cfg(any(target_os = "linux", target_os = "windows"))]
    {
        if cuda_available() {
            tracing::info!("CUDA GPU acceleration available");
            return AcceleratorType::CUDA;
        }
    }
    
    tracing::info!("Using CPU for inference");
    AcceleratorType::CPU
}

#[cfg(target_os = "macos")]
fn metal_available() -> bool {
    true
}

#[cfg(any(target_os = "linux", target_os = "windows"))]
fn cuda_available() -> bool {
    std::process::Command::new("nvidia-smi")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}
