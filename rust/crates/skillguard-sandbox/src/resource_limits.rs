/// Resource limits for sandboxed execution.
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    /// Maximum memory in bytes.
    pub max_memory_bytes: usize,
    /// Maximum fuel (instruction count proxy).
    pub max_fuel: u64,
    /// Timeout in seconds (via epoch interruption).
    pub timeout_seconds: u64,
    /// Maximum output size in bytes.
    pub max_output_bytes: usize,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_bytes: 512 * 1024 * 1024, // 512 MB
            max_fuel: 1_000_000_000,             // ~1B instructions
            timeout_seconds: 30,
            max_output_bytes: 1024 * 1024, // 1 MB
        }
    }
}

impl ResourceLimits {
    pub fn restricted() -> Self {
        Self {
            max_memory_bytes: 64 * 1024 * 1024, // 64 MB
            max_fuel: 100_000_000,
            timeout_seconds: 10,
            max_output_bytes: 256 * 1024, // 256 KB
        }
    }
}
