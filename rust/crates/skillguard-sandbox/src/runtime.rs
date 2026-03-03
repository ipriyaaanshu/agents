use anyhow::Result;
use wasmtime::{Config, Engine};

/// Shared Wasmtime engine configuration for all sandbox executions.
pub struct SandboxRuntime {
    engine: Engine,
}

impl SandboxRuntime {
    /// Create a new sandbox runtime with secure defaults.
    pub fn new() -> Result<Self> {
        let mut config = Config::new();
        config.wasm_component_model(true);
        config.epoch_interruption(true);
        config.consume_fuel(true);

        // Security: disable features that could be used for escape
        config.wasm_threads(false);
        config.wasm_simd(true);
        config.wasm_bulk_memory(true);

        let engine = Engine::new(&config)?;
        Ok(Self { engine })
    }

    /// Get a reference to the underlying engine.
    pub fn engine(&self) -> &Engine {
        &self.engine
    }
}

impl Default for SandboxRuntime {
    fn default() -> Self {
        Self::new().expect("Failed to create sandbox runtime")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_creation() {
        let runtime = SandboxRuntime::new();
        assert!(runtime.is_ok());
    }
}
