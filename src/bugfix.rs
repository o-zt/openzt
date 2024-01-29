
use std::arch::global_asm;

use tracing::info;

use crate::debug_dll::{get_from_memory, save_to_protected_memory, patch_nop};

const ZOOWALL_MAP_EDGE_CRASH_ADDRESS: u32 = 0x050b260;

const ZOOFENCE_ONE_TILE_FROM_MAP_EDGE_CRASH_ADDRESS_1: u32 = 0x4a1fc0;
const ZOOFENCE_ONE_TILE_FROM_MAP_EDGE_CRASH_ADDRESS_2: u32 = 0x4a1fe7;
// const ZOOFENCE_ONE_TILE_FROM_MAP_EDGE_CRASH_ADDRESS_1: u32 = ;

pub fn init() {
    fix_zoowall_map_edge_crash();
    fix_fence_one_tile_from_map_edge_crash()
}


fn fix_zoowall_map_edge_crash() {
    // We change a jump address to fix a bug trying access a null pointer
    save_to_protected_memory(ZOOWALL_MAP_EDGE_CRASH_ADDRESS, 0xfffffcfeu32 as i32);
}

fn fix_fence_one_tile_from_map_edge_crash() {
    // This changes an if statement to cover the entire inner loop
    save_to_protected_memory::<u8>(ZOOFENCE_ONE_TILE_FROM_MAP_EDGE_CRASH_ADDRESS_1, 0x45);
    // The above change makes the second if statement redundant so we can add in a check for the null pointer
    save_to_protected_memory::<u8>(ZOOFENCE_ONE_TILE_FROM_MAP_EDGE_CRASH_ADDRESS_2, 0x85);
    save_to_protected_memory::<u8>(ZOOFENCE_ONE_TILE_FROM_MAP_EDGE_CRASH_ADDRESS_2 + 1, 0xC0);
    patch_nop(ZOOFENCE_ONE_TILE_FROM_MAP_EDGE_CRASH_ADDRESS_2 + 2);
}

// Leaving this in incase future bugfixes require inline assembly

// pub mod fence_crash_asm {
//     global_asm!(r#"
//         .global fence_crash_trmp
//       :
//         call DWORD PTR ds:0x40fa92
//         TEST EAX, EAX
//         JZ 0x01
//         JMP DWORD PTR 
//         JMP DWORD PTR ds:<past original if>

//         jmp back to original if
//     "#);
// }

// the symbols `foo` and `bar` are global, no matter where
// `global_asm!` was used.
// extern "C" {
//     fn fence_crash_trmp();
// }