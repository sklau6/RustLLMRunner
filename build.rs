fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    
    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-link-lib=framework=Metal");
        println!("cargo:rustc-link-lib=framework=MetalKit");
        println!("cargo:rustc-link-lib=framework=Foundation");
    }
    
    #[cfg(any(target_os = "linux", target_os = "windows"))]
    {
        if let Ok(cuda_path) = std::env::var("CUDA_PATH") {
            println!("cargo:rustc-link-search=native={}/lib64", cuda_path);
            println!("cargo:rustc-link-lib=cudart");
            println!("cargo:rustc-link-lib=cublas");
        }
    }
}
