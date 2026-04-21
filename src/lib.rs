use anyhow::Result;
use std::collections::BTreeMap;
use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::LazyLock;
use std::sync::Mutex;

pub mod hook;
use hook::Hook;

pub static ORIGINAL_FUNCTIONS: LazyLock<Mutex<BTreeMap<String, (usize, usize)>>> =
    LazyLock::new(|| Mutex::new(BTreeMap::new()));

// thread counters
pub static HOOK_COUNTER: AtomicUsize = AtomicUsize::new(0);
pub static DISABLE: AtomicBool = AtomicBool::new(false);

pub struct Hooks {
    hooks: Vec<Hook>,
}

impl Hooks {
    pub fn new(hooks: &[Hook]) -> Result<Self> {
        min_hook_rs::initialize()?;
        Ok(Self {
            hooks: hooks.to_vec(),
        })
    }
    pub fn enable(&self) -> Result<()> {
        log::info!("Enabling!");

        for hook in self.hooks.iter() {
            hook.enable()?;
        }

        Ok(())
    }
    pub fn disable(&self) -> Result<()> {
        log::info!("Disabling!");

        // Set global disable for hooks
        DISABLE.store(true, Ordering::Relaxed);

        while HOOK_COUNTER.load(Ordering::Relaxed) != self.hooks.len() {
            log::info!("Waiting to unhook");
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        min_hook_rs::uninitialize()?;

        Ok(())
    }
}

pub fn get_function<T>(function: &str) -> Option<(T, usize)>
where
    T: Copy,
{
    let map = &(ORIGINAL_FUNCTIONS.lock().unwrap());
    if let Some((original, target)) = map.get(function) {
        let func: T = unsafe { std::mem::transmute_copy(&(*original as *const c_void)) };
        return Some((func, *target));
    } else {
        None
    }
}

pub fn store_function(name: String, function: *mut c_void, target: *mut c_void) {
    let map = &mut (ORIGINAL_FUNCTIONS.lock().unwrap());
    map.insert(name, (function as usize, target as usize));
}

pub fn increment_counter() {
    HOOK_COUNTER.fetch_add(1, Ordering::Relaxed);
}

pub fn disable_hook(target: usize, name: &'static str) {
    let target = target as *mut c_void;

    if !target.is_null() {
        min_hook_rs::hook::disable_hook(target).map_err(|err| {
            log::error!("Failed to disable hook {name}");
            return;
        });
        log::info!("Disabled {name}");

        min_hook_rs::hook::remove_hook(target).map_err(|err| {
            log::error!("Failed to disable hook {name}");
            return;
        });
        log::info!("Removed {name}");
    } else {
        log::error!("Hook was null! {name}")
    }
}
