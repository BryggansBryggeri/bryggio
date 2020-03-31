#[cfg(target_arch = "x86_64")]
pub(crate) mod dummy;
#[cfg(target_arch = "arm")]
pub(crate) mod rbpi;
