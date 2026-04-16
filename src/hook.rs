use anyhow::{anyhow, Result};
use min_hook_rs::hook::*;
use std::ffi::c_void;

use crate::*;

#[derive(Debug, Clone)]
pub struct Hook {
    module: &'static str,
    name: &'static str,
    detour: *mut c_void,
    target: Option<*mut c_void>,
}

impl Hook {
    pub const fn new(module: &'static str, name: &'static str, detour: *mut c_void) -> Self {
        Self {
            module,
            name,
            detour,
            target: None,
        }
    }
    pub fn enable(&mut self) -> Result<()> {
        let (trampoline, target) = create_hook_api(self.module, self.name, self.detour)?;

        store_function(self.name.to_string(), trampoline);

        enable_hook(target)?;
        log::info!("Enabled {}", self.name);

        self.target = Some(target);

        Ok(())
    }
    pub fn disable(&self) -> Result<()> {
        if let Some(target) = self.target {
            if !target.is_null() {
                disable_hook(target)?;
                log::info!("Disabled {}", self.name);
            } else {
                return Err(anyhow!("Failed to disable for {}", self.name));
            }
        } else {
            log::warn!("Target for {} is {:?}", self.name, self.target);
        }

        Ok(())
    }
    pub fn remove(&self) -> Result<()> {
        if let Some(target) = self.target {
            if !target.is_null() {
                remove_hook(target)?;
                log::info!("Removed {}", self.name);
            } else {
                return Err(anyhow!("Failed to remove for {}", self.name));
            }
        } else {
            log::warn!("Target for {} is {:?}", self.name, self.target);
        }
        Ok(())
    }
}
