//! larp-jv-native — Native libraries for larp.jv

pub mod mojoexec;
pub mod jvm_hooks;
pub mod jnidispatch;
pub mod unpack200;
pub mod gl4es;
pub mod freetype;

/// Initialize all native libraries
pub fn init() {
    mojoexec::init();
    jvm_hooks::init();
    jnidispatch::init();
    unpack200::init();
    gl4es::init();
    freetype::init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        init();
    }
}