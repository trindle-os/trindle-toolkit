//! Keeping the GPU drawer's memory down: freed heap is handed back to the system once the drawer has started.

pub(crate) fn settle() {
    // SAFETY: a plain call that returns unused heap to the system.
    #[cfg(target_env = "gnu")]
    unsafe {
        libc::malloc_trim(0)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settling_leaves_the_program_working() {
        settle();
        let v: Vec<u8> = (0..100_000).map(|i| i as u8).collect();
        assert_eq!(v[99_999], 99_999u32 as u8);
    }
}
