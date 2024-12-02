#![cfg(feature="__rt__")]

use crate::util::unix_timestamp;
use ohkami_lib::time::{UTCDateTime, IMF_FIXDATE_LEN};


/// SAFETY: A callee ONLY uses the returned reference for at most 0.5 secs. 
pub unsafe fn imf_fixdate_now() -> &'static str {
    use std::sync::atomic::{AtomicPtr, Ordering};
    use std::sync::Once;
    use std::time::Duration;
    use std::cell::UnsafeCell;

    static RCU: RCU = RCU {
        a: UnsafeCell::new([0; IMF_FIXDATE_LEN]),
        b: UnsafeCell::new([0; IMF_FIXDATE_LEN])
    };
    struct RCU {
        a: UnsafeCell<[u8; IMF_FIXDATE_LEN]>,
        b: UnsafeCell<[u8; IMF_FIXDATE_LEN]>
    }
    unsafe impl Send for RCU {}
    unsafe impl Sync for RCU {}

    const A: *mut [u8; IMF_FIXDATE_LEN] = RCU.a.get();
    const B: *mut [u8; IMF_FIXDATE_LEN] = RCU.b.get();

    static NOW: AtomicPtr<[u8; IMF_FIXDATE_LEN]> = AtomicPtr::new(A);

    static INIT: Once = Once::new();
    INIT.call_once(|| {
        unsafe {**NOW.as_ptr() = UTCDateTime::from_unix_timestamp(unix_timestamp()).into_imf_fixdate()}        
        std::thread::spawn(|| loop {std::thread::sleep(Duration::from_millis(500));
            crate::DEBUG!("NOW: {}", std::str::from_utf8(&**NOW.as_ptr()).unwrap());
            let next = if (*NOW.as_ptr()) == A {B} else {A};
            unsafe {*next = UTCDateTime::from_unix_timestamp(unix_timestamp()).into_imf_fixdate()};
            NOW.store(next, Ordering::Release);
        });
    });
    
    // SAFETY: into_imf_fixdate() always generates valid UTF-8 bytes
    unsafe {std::str::from_utf8_unchecked(&*NOW.load(Ordering::Acquire))}
}
