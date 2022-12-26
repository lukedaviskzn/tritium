use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Uid(u64);

impl Uid {
    pub fn new() -> Uid {
        static mut LAST_NODE_ID: AtomicU64 = AtomicU64::new(0);
        let id = unsafe { LAST_NODE_ID.fetch_add(1, Ordering::Relaxed) };
        Uid(id)
    }
}

pub trait AsAny {
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}
