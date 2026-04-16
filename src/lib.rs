use anyhow::Result;
use min_hook_rs::hook::*;
use std::collections::BTreeMap;
use std::ffi::c_void;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::LazyLock;
use std::sync::Mutex;

pub mod hook;
use hook::Hook;

pub static ORIGINAL_FUNCTIONS: LazyLock<Mutex<BTreeMap<String, usize>>> =
    LazyLock::new(|| Mutex::new(BTreeMap::new()));

// thread counters
pub static HOOK_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub struct Hooks {
    hooks: Vec<Hook>,
}

impl Hooks {
    pub fn new(hooks: &[Hook]) -> Result<Self> {
        initialize()?;
        Ok(Self {
            hooks: hooks.to_vec(),
        })
    }
    pub fn enable(&mut self) -> Result<()> {
        log::info!("Enabling!");

        for hook in self.hooks.iter_mut() {
            hook.enable()?;
        }

        Ok(())
    }
    pub fn disable(&self) -> Result<()> {
        log::info!("Disabling!");
        for hook in self.hooks.iter() {
            hook.disable()?;
        }

        // waiting for ref counter to be 0 so we can remove hooks
        // as to not remove hook while thread is inside of it .
        while HOOK_COUNTER.load(Ordering::Relaxed) != 0 {
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        log::info!("Removing Hooks!");
        for hook in self.hooks.iter() {
            hook.remove()?;
        }

        uninitialize()?;

        Ok(())
    }
}

pub fn get_function<T>(function: &str) -> Option<T>
where
    T: Copy,
{
    let map = &(ORIGINAL_FUNCTIONS.lock().unwrap());
    if let Some(original) = map.get(function) {
        let func: T = unsafe { std::mem::transmute_copy(&(*original as *const c_void)) };
        return Some(func);
    } else {
        None
    }
}

pub fn store_function(name: String, function: *mut c_void) {
    let map = &mut (ORIGINAL_FUNCTIONS.lock().unwrap());
    map.insert(name, function as usize);
}

pub fn increment_counter(counter: &AtomicUsize) {
    let count = counter.load(Ordering::Relaxed);
    counter.store(count + 1, Ordering::Relaxed);
}

pub fn decrement_counter(counter: &AtomicUsize) {
    let count = counter.load(Ordering::Relaxed);
    counter.store(count - 1, Ordering::Relaxed);
}
