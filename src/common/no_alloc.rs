#[macro_export]
macro_rules! no_alloc {
    ($s: stmt) => {
        let no_alloc = &$crate::simple_alloc::IN_NO_ALLOC_SECTION;
        let prev_no_alloc = no_alloc.load(core::sync::atomic::Ordering::Relaxed);
        no_alloc.store(true, core::sync::atomic::Ordering::Relaxed);
        $s
        no_alloc.store(prev_no_alloc, core::sync::atomic::Ordering::Relaxed);
    };
}
