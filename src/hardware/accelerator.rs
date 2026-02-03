use anyhow::Result;
use crate::hardware::AcceleratorType;

pub struct Accelerator {
    accel_type: AcceleratorType,
    device_count: usize,
}

impl Accelerator {
    pub fn new() -> Result<Self> {
        let accel_type = crate::hardware::detect_accelerator();
        let device_count = Self::get_device_count(accel_type);
        
        Ok(Self {
            accel_type,
            device_count,
        })
    }
    
    fn get_device_count(accel_type: AcceleratorType) -> usize {
        match accel_type {
            AcceleratorType::CPU => 1,
            AcceleratorType::CUDA => {
                #[cfg(any(target_os = "linux", target_os = "windows"))]
                {
                    Self::get_cuda_device_count()
                }
                #[cfg(not(any(target_os = "linux", target_os = "windows")))]
                {
                    0
                }
            }
            AcceleratorType::Metal => {
                #[cfg(target_os = "macos")]
                {
                    1
                }
                #[cfg(not(target_os = "macos"))]
                {
                    0
                }
            }
            AcceleratorType::ROCm => 0,
        }
    }
    
    #[cfg(any(target_os = "linux", target_os = "windows"))]
    fn get_cuda_device_count() -> usize {
        std::process::Command::new("nvidia-smi")
            .args(&["--query-gpu=count", "--format=csv,noheader"])
            .output()
            .ok()
            .and_then(|output| {
                String::from_utf8(output.stdout)
                    .ok()
                    .and_then(|s| s.trim().parse().ok())
            })
            .unwrap_or(0)
    }
    
    pub fn get_type(&self) -> AcceleratorType {
        self.accel_type
    }
    
    pub fn device_count(&self) -> usize {
        self.device_count
    }
    
    pub fn is_gpu_available(&self) -> bool {
        matches!(self.accel_type, AcceleratorType::CUDA | AcceleratorType::Metal | AcceleratorType::ROCm)
    }
    
    pub fn get_recommended_layers(&self) -> i32 {
        if self.is_gpu_available() {
            -1
        } else {
            0
        }
    }
}

impl Default for Accelerator {
    fn default() -> Self {
        Self::new().unwrap()
    }
}
