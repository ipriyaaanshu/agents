use crate::capability::CapabilityGrants;
use crate::resource_limits::ResourceLimits;
use crate::runtime::SandboxRuntime;
use anyhow::Result;
use skillguard_core::{SkillResult, SkillStatus};
use std::path::Path;
use wasmtime::{Store, StoreLimits, StoreLimitsBuilder};

struct SandboxState {
    limits: StoreLimits,
    wasi: wasmtime_wasi::WasiCtx,
    table: wasmtime::component::ResourceTable,
}

impl wasmtime_wasi::WasiView for SandboxState {
    fn ctx(&mut self) -> &mut wasmtime_wasi::WasiCtx {
        &mut self.wasi
    }
    fn table(&mut self) -> &mut wasmtime::component::ResourceTable {
        &mut self.table
    }
}

/// Executes WASM skill components in a sandboxed environment.
pub struct SandboxExecutor {
    runtime: SandboxRuntime,
}

impl SandboxExecutor {
    pub fn new() -> Result<Self> {
        Ok(Self {
            runtime: SandboxRuntime::new()?,
        })
    }

    /// Execute a WASM component with the given capabilities and limits.
    pub fn execute(
        &self,
        wasm_path: &Path,
        action: &str,
        params: &serde_json::Value,
        capabilities: &CapabilityGrants,
        limits: &ResourceLimits,
    ) -> Result<SkillResult> {
        let engine = self.runtime.engine();

        // Build WASI context with capability-restricted preopens
        let mut wasi_builder = wasmtime_wasi::WasiCtxBuilder::new();

        // Add readonly directories
        for dir in &capabilities.readonly_dirs {
            if dir.exists() {
                let host_dir = wasmtime_wasi::DirPerms::READ;
                let host_file = wasmtime_wasi::FilePerms::READ;
                wasi_builder.preopened_dir(
                    dir,
                    dir.to_string_lossy().as_ref(),
                    host_dir,
                    host_file,
                )?;
            }
        }

        // Add read-write directories
        for dir in &capabilities.readwrite_dirs {
            if dir.exists() {
                let host_dir = wasmtime_wasi::DirPerms::all();
                let host_file = wasmtime_wasi::FilePerms::all();
                wasi_builder.preopened_dir(
                    dir,
                    dir.to_string_lossy().as_ref(),
                    host_dir,
                    host_file,
                )?;
            }
        }

        // Filter environment variables
        for var in &capabilities.allowed_env_vars {
            if let Ok(val) = std::env::var(var) {
                wasi_builder.env(var, &val);
            }
        }

        // Pass action and params as args
        wasi_builder.args(&[action, &params.to_string()]);

        let wasi = wasi_builder.build();
        let table = wasmtime::component::ResourceTable::new();

        let store_limits = StoreLimitsBuilder::new()
            .memory_size(limits.max_memory_bytes)
            .build();

        let state = SandboxState {
            limits: store_limits,
            wasi,
            table,
        };

        let mut store = Store::new(engine, state);
        store.limiter(|s| &mut s.limits);
        store.set_fuel(limits.max_fuel)?;
        store.epoch_deadline_trap();

        // Load the WASM component
        let component = wasmtime::component::Component::from_file(engine, wasm_path)?;

        // Create linker with WASI
        let mut linker = wasmtime::component::Linker::new(engine);
        wasmtime_wasi::add_to_linker_sync(&mut linker)?;

        // Instantiate and run
        let instance = linker.instantiate(&mut store, &component)?;

        // Try to call the exported "execute" function
        let func = instance
            .get_func(&mut store, "execute")
            .ok_or_else(|| anyhow::anyhow!("WASM component has no 'execute' export"))?;

        // Call with action and params as string args
        let mut results = vec![wasmtime::component::Val::String("".into())];
        let args = [
            wasmtime::component::Val::String(action.into()),
            wasmtime::component::Val::String(params.to_string()),
        ];
        func.call(&mut store, &args, &mut results)?;

        // Parse the result
        if let Some(wasmtime::component::Val::String(result_str)) = results.first() {
            match serde_json::from_str::<SkillResult>(result_str) {
                Ok(result) => Ok(result),
                Err(_) => Ok(SkillResult::success(serde_json::Value::String(
                    result_str.to_string(),
                ))),
            }
        } else {
            Ok(SkillResult {
                status: SkillStatus::Success,
                data: None,
                error_message: None,
                metadata: Default::default(),
            })
        }
    }
}

impl Default for SandboxExecutor {
    fn default() -> Self {
        Self::new().expect("Failed to create sandbox executor")
    }
}
