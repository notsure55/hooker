use anyhow::{anyhow, Result};
use min_hook_rs::hook::*;
use std::ffi::c_void;

use crate::*;

#[derive(Debug, Clone)]
pub struct Ptr {
    name: String,
    target: *mut c_void,
    detour: *mut c_void,
}

impl Ptr {
    pub fn new(name: &str, target: *mut c_void, detour: *mut c_void) -> Self {
        Self {
            name: String::from(name),
            target,
            detour,
        }
    }
    pub fn enable(&self) -> Result<()> {
        let trampoline = create_hook(self.target, self.detour)?;

        store_function(self.name.clone(), trampoline, self.target);

        enable_hook(self.target)?;

        log::info!("Enabled {}", self.name);

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum Hook {
    Api(Api),
    Ptr(Ptr),
}

impl Hook {
    pub fn enable(&self) -> Result<()> {
        match self {
            Hook::Api(api) => api.enable()?,
            Hook::Ptr(ptr) => ptr.enable()?,
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Api {
    module: &'static str,
    name: &'static str,
    detour: *mut c_void,
}

impl Api {
    pub const fn new(module: &'static str, name: &'static str, detour: *mut c_void) -> Self {
        Self {
            module,
            name,
            detour,
        }
    }
    pub fn enable(&self) -> Result<()> {
        let (trampoline, target) = create_hook_api(self.module, self.name, self.detour)?;

        store_function(self.name.to_string(), trampoline, target);

        enable_hook(target)?;

        log::info!("Enabled {}", self.name);

        Ok(())
    }
}
