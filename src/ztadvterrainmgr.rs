use std::{fmt, fmt::Display};

use tracing::info;

use crate::{
    add_to_command_register,
    debug_dll::{get_from_memory, get_string_from_memory},
    console::CommandError,
};

const GLOBAL_ZTADVTERRAINMGR_ADDRESS: u32 = 0x00638058;
const BFTERRAINTYPEINFO_SIZE: usize = 0x30;

#[derive(Debug)]
#[repr(C)]
struct ZTAdvTerrainMgr_raw {
    vtable: u32,
    unknown_u32_1: u32,
    unknown_u32_2: u32,
    unknown_u32_3: u32,
    bf_terrain_type_info_array_start: u32,
    bf_terrain_type_info_array_end: u32,
    bf_terrain_type_info_buffer_end: u32,
    // Total size is 0x1dc
}

struct ZTAdvTerrainMgr {
    bf_terrain_type_info_array: Vec<BFTerrainTypeInfo>,
}

impl From<ZTAdvTerrainMgr_raw> for ZTAdvTerrainMgr {
    fn from(raw: ZTAdvTerrainMgr_raw) -> Self {
        info!(
            "Reading terrain types from {:#x} to {:#x}",
            raw.bf_terrain_type_info_array_start, raw.bf_terrain_type_info_array_end
        );
        let mut bf_terrain_type_info_array = Vec::new();
        let mut current_bf_terrain_type_info_address = raw.bf_terrain_type_info_array_start;
        while current_bf_terrain_type_info_address < raw.bf_terrain_type_info_array_end {
            bf_terrain_type_info_array.push(read_bfterraintypeinfo_from_memory(
                current_bf_terrain_type_info_address,
            ));
            current_bf_terrain_type_info_address += BFTERRAINTYPEINFO_SIZE as u32;
        }
        ZTAdvTerrainMgr {
            bf_terrain_type_info_array,
        }
    }
}

impl Display for BFTerrainTypeInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "BFTerrainTypeInfo {{ vtable: {:#x}\n type_id: {}\n cost: {}\n blend: {}\n water: {}\n unknown_ptr: {:#x}\n unknown_u32_1: {:#x}\n unknown_u32_2: {:#x}\n help_id: {}\n icon_string: {}\n }}",
            self.vtable,
            self.type_id,
            self.cost,
            self.blend,
            self.water,
            self.unknown_ptr,
            self.unknown_u32_6,
            self.unknown_u32_7,
            self.help_id,
            get_string_from_memory(self.icon_string_start)
        )
    }
}

#[derive(Debug)]
#[repr(C)]
struct BFTerrainTypeInfo {
    vtable: u32,
    type_id: u32,
    cost: f32,
    blend: u32,
    water: u32,
    unknown_ptr: u32,
    unknown_u32_6: u32,
    unknown_u32_7: u32,
    help_id: u32,
    icon_string_start: u32,
    icon_string_end: u32,
    icon_string_buffer_end: u32,
}

fn read_ztadvterrainmgr_raw_from_memory() -> ZTAdvTerrainMgr_raw {
    get_from_memory(get_from_memory::<u32>(GLOBAL_ZTADVTERRAINMGR_ADDRESS))
}

fn read_ztadvterrainmgr_from_memory() -> ZTAdvTerrainMgr {
    ZTAdvTerrainMgr::from(read_ztadvterrainmgr_raw_from_memory())
}

fn read_bfterraintypeinfo_from_memory(address: u32) -> BFTerrainTypeInfo {
    get_from_memory(address)
}

fn command_get_bfterraintypeinfo(_args: Vec<&str>) -> Result<String, CommandError> {
    let ztadvterrainmgr = read_ztadvterrainmgr_from_memory();
    info!(
        "Found {} BFTerrainTypeInfo",
        ztadvterrainmgr.bf_terrain_type_info_array.len()
    );
    let mut string_array = Vec::new();
    for bfterraintypeinfo in ztadvterrainmgr.bf_terrain_type_info_array {
        string_array.push(bfterraintypeinfo.to_string());
    }
    Ok(string_array.join("\n"))
}

pub fn init() {
    add_to_command_register(
        "list_bfterraintypeinfo".to_string(),
        command_get_bfterraintypeinfo,
    );
}

// #[hook_module("zoo.exe")]
// mod zt_adv_terrain_mgr {
//     #[hook(unsafe extern "thiscall" ZTAdvTerrainMgr_loadTextures, offset = 0x001224b9)]
//     fn zoo_zt_adv_terrain_mgr_load_textures(this_ptr: u32) -> u32 {
//         let return_value = unsafe { ZTAdvTerrainMgr_loadTextures.call(this_ptr) };
//         return_value
//     }

//     #[hook(unsafe extern "thiscall" BFTerrainTypeInfo_initialize, offset = 0x00123c58)]
//     fn zoo_bf_terrain_type_info_initialize(this_ptr: u32, config_ptr: u32, name: u32) -> u32 {
//         let return_value = unsafe { BFTerrainTypeInfo_initialize.call(this_ptr, config_ptr, name) };
//         return_value
//     }


//     #[hook(unsafe extern "thiscall" BFMap_paintCell, offset = 0x000f8fd8)]
//     fn zoo_bf_map_paint_cell(this_ptr: u32, bf_terrain_type_info_ptr: u32, param: bool) -> u32 {
//         info!(
//             "BFMap::paintCell({:X}, {:X}, {} -> {:X})",
//             this_ptr,
//             bf_terrain_type_info_ptr,
//             param,
//             get_from_memory::<u32>(bf_terrain_type_info_ptr)
//         );
//         let return_value =
//             unsafe { BFMap_paintCell.call(this_ptr, bf_terrain_type_info_ptr, param) };
//         info!(
//             "BFMap::paintCell({:X}, {:X}, {}) -> {:X}",
//             this_ptr, bf_terrain_type_info_ptr, param, return_value
//         );
//         return_value
//     }

//     #[hook(unsafe extern "thiscall" BFMap_paintCell2, offset = 0x000f17e0)]
//     fn zoo_bf_tile_set_terrain_type(this_ptr: u32, bf_terrain_type_info_ptr: u32) -> u32 {
//         info!("BFTile::setTerrainType({:X}, {:X} -> {:X})", this_ptr, bf_terrain_type_info_ptr, get_from_memory(bf_terrain_type_info_ptr));
//         let return_value = unsafe { BFMap_paintCell.call(this_ptr, bf_terrain_type_info_ptr) };
//         info!("BFTile::setTerrainType({:X}, {:X}) -> {:X}", this_ptr, bf_terrain_type_info_ptr, return_value);
//         return_value
//     }
// }
