//! Heap tuning for a software GL (sim-long sl-1): llvmpipe keeps the pixels
//! of a texture on the heap, and glibc keeps a freed arena for later until
//! it is trimmed. Both calls are glibc only; elsewhere they do nothing.

#[cfg(all(target_os = "linux", target_env = "gnu"))]
#[allow(unsafe_code, reason = "two glibc calls that take and return numbers")]
mod imp {
    /// Sets a fixed mmap threshold, so that freed textures go back to the
    /// system. Call it before any thread starts.
    pub fn tune() {
        // SAFETY: mallopt takes two integers and touches only glibc's own state; it runs before any thread starts.
        unsafe { libc::mallopt(libc::M_MMAP_THRESHOLD, 64 << 10) };
    }

    /// Gives freed heap back to the system.
    pub fn trim() {
        // SAFETY: malloc_trim takes one integer and is safe to call from any thread.
        unsafe { libc::malloc_trim(0) };
    }
}

#[cfg(not(all(target_os = "linux", target_env = "gnu")))]
mod imp {
    pub fn tune() {}
    pub fn trim() {}
}

pub use imp::{trim, tune};
