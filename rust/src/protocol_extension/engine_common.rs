// const POWER_SMALLEST: i32 = 5;
// const POWER_LARGEST: i32 = 200;
// const MAX_SLAB_CLASSES: i32 = POWER_LARGEST + 1;

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct ENGINE_HANDLE {
    interface: u64
}
