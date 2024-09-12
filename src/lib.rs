#![feature(let_chains)]
#![allow(dead_code)]

use std::{net::TcpStream, sync::Mutex};

use bf_configparser::ini::Ini;
use retour_utils::hook_module;
use tracing::{error, info, Level};

/// Reimplementation of the BFRegistry, a vanilla system used to store pointers to the ZT*Mgr classes. In theory this 
/// allowed customization via zoo.ini, but in practice it appears unused.
mod bfregistry;

/// Hooks into the vanilla game's logging system to re-log messages with the default OpenZT logger.
mod capture_ztlog;

/// Basic development console, includes a server that listens for a client connection to recieve commands from, 
/// functions for registering commands with a function callback and hooks so that a command is run every game update
mod console;

/// Commands and functions for reading entities and entity types from the ZTWorldMgr class
mod ztworldmgr;

mod resource_manager;

/// Reading and changing the state of the UI, contains hooks for UI elements and some basic UI manipulation functions.
mod ztui;

/// Assembly patches and functions to fix bugs in the vanilla game.
/// 
/// Currently fixes a crash when a maintenance worker tries to fix a 
/// fence 1 tile away from the edge of the map, and a bug where the 
/// game crashes if a zoo wall that is one tile away from the edge 
/// of the map is deleted.
mod bugfix;

/// Methods for reading the vanilla ZTAdvTerrainMgr class, which contains information about terrain types.
mod ztadvterrainmgr;

/// Reimplementation of vanilla handling of Expansion Packs, including the ability to define custom expansions.
/// 
/// Default behaviour adds in an expansion called "Custom Content" which includes all non-vanilla entities.
/// Expanding the Expansion dropdown is also handled here.
mod expansions;

/// Reimplementation of the vanilla BFApp::loadString, has functions to add a string to the OpenZT string registry, 
/// will fallback to the vanilla BFApp::loadString if the string is not found in the registry.
mod string_registry;

//TODO: Combine debug, common and binary_parsing into a util module
mod common;

/// Helper methods for parsing binary data, including reading and writing binary data to and from buffers.
mod binary_parsing;

/// ZTAF Animation file format parsing, writing and some modification methods.
/// 
/// Based on documentation at <https://github.com/jbostoen/ZTStudio/wiki/ZT1-Graphics-Explained>
mod animation;

/// Structs that mirror ZT Entity types and their properties. Currently there are many missing fields.
mod bfentitytype;

/// ztgamemgr module has commands to interact with the live zoo stats such as cash, num animals, species, guests, etc. via the vanilla ZTGameMgr class.
mod ztgamemgr;

/// Patches in the current OpenZT build version into the game's version string.
mod version;

/// OpenZT mod structs
mod mods;

#[cfg(target_os = "windows")]
use winapi::um::winnt::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH, DLL_THREAD_ATTACH, DLL_THREAD_DETACH};

use crate::{
    console::{add_to_command_register, zoo_console},
    debug_dll::{command_get_setting, command_set_setting, command_show_settings},
};

#[cfg(not(target_os = "windows"))]
mod linux {
    const DLL_PROCESS_DETACH: u32 = 0;
    const DLL_PROCESS_ATTACH: u32 = 1;
    const DLL_THREAD_ATTACH: u32 = 2;
    const DLL_THREAD_DETACH: u32 = 3;
}

mod debug_dll;
mod load_ini;

#[no_mangle]
pub fn dll_first_load() {
    let Ok(stream) = TcpStream::connect("127.0.0.1:1492") else {
        info!("Failed to connect to log stream");
        return;
    };

    let subscriber = tracing_subscriber::fmt().with_writer(Mutex::new(stream)).with_max_level(Level::INFO).finish();

    if tracing::subscriber::set_global_default(subscriber).is_err() {
        info!("Failed to set global default subscriber, logging may not function");
    }

    info!("openzt.dll Loaded");
}

#[hook_module("zoo.exe")]
mod zoo_ini {
    use crate::load_debug_settings_from_ini;

    #[hook(unsafe extern "cdecl" LoadDebugSettingsFromIniHook, offset = 0x00179f4c)]
    fn load_debug_settings_from_ini_detour() {
        load_debug_settings_from_ini();
    }
}

#[no_mangle]
extern "system" fn DllMain(module: u8, reason: u32, _reserved: u8) -> i32 {
    match reason {
        DLL_PROCESS_ATTACH => {
            dll_first_load();
            info!("DllMain: DLL_PROCESS_ATTACH: {}, {} {}", module, reason, _reserved);

            // Initialize stable modules
            resource_manager::init();
            expansions::init();
            string_registry::init();
            bugfix::init();
            version::init();

            if cfg!(feature = "ini") {
                info!("Feature 'ini' enabled");
                if unsafe { zoo_ini::init_detours() }.is_err() {
                    error!("Failed to initialize ini detours");
                };
                add_to_command_register("list_settings".to_owned(), command_show_settings);
                add_to_command_register("get_setting".to_owned(), command_get_setting);
                add_to_command_register("set_setting".to_owned(), command_set_setting);
            }
            if cfg!(feature = "bf_registry") {
                use crate::bfregistry;
                info!("Feature 'bf_registry' enabled");
                bfregistry::init();
            }

            if cfg!(feature = "capture_ztlog") {
                use crate::capture_ztlog;
                info!("Feature 'capture_ztlog' enabled");
                capture_ztlog::init();
            }

            if cfg!(feature = "ztui") {
                info!("Feature 'ztui' enabled");
                ztui::init();
            }

            if cfg!(feature = "console") {
                info!("Feature 'console' enabled");
                zoo_console::init();
            }

            if cfg!(feature = "experimental") {
                info!("Feature 'experimental' enabled");
                ztadvterrainmgr::init();
                ztworldmgr::init();
                bfentitytype::init();
                ztgamemgr::init();
                // unsafe { zoo_misc::init_detours() };
            }
        }
        DLL_PROCESS_DETACH => {
            info!("DllMain: DLL_PROCESS_DETACH: {}, {} {}", module, reason, _reserved);
        }
        DLL_THREAD_ATTACH => {
            info!("DllMain: DLL_THREAD_ATTACH: {}, {} {}", module, reason, _reserved);
        }
        DLL_THREAD_DETACH => {
            info!("DllMain: DLL_THREAD_DETACH: {}, {} {}", module, reason, _reserved);
        }
        _ => {
            info!("DllMain: Unknown: {}, {} {}", module, reason, _reserved);
        }
    }
    1
}

#[no_mangle]
extern "C" fn dll_ini_debug_log() {
    debug_dll::log_debug_ini_memory_values();
}

fn load_debug_settings_from_ini() {
    debug_dll::debug_logger("load_debug_settings_from_ini");
    debug_dll::log_exe_location_memory_value();
    debug_dll::log_debug_ini_memory_values();
    let mut base_path = debug_dll::get_base_path();
    base_path.push("zoo.ini");
    let debug_settings = load_ini::load_debug_settings(base_path.as_path());
    debug_dll::debug_logger("Saving debug ini settings");
    debug_dll::save_debug_settings(debug_settings);
    debug_dll::log_debug_ini_memory_values();
}

#[no_mangle]
pub fn patch_load_debug_ini_call() {
    debug_dll::debug_logger(&format!("load_debug_settings_from_ini {:p}", load_debug_settings_from_ini as *const ()));
    debug_dll::debug_logger(&format!("load_debug_settings_from_ini (u32) {}", load_debug_settings_from_ini as u32));
    debug_dll::get_code_from_memory(debug_dll::DEBUG_INI_LOAD_CALL_ADDRESS, 0x10);
    debug_dll::patch_call(debug_dll::DEBUG_INI_LOAD_CALL_ADDRESS, load_debug_settings_from_ini as u32);
}

#[no_mangle]
extern "C" fn patch_load_int_from_ini_call() {
    debug_dll::debug_logger(&format!("load_int_from_ini {:p}", load_int_from_ini as *const ()));
    debug_dll::patch_calls(debug_dll::LOAD_INT_FROM_INI_ADDRESS_ARRAY_SUBSET.to_vec(), load_int_from_ini as u32);
    debug_dll::patch_nops_series(debug_dll::LOAD_INT_FROM_INI_ADDRESS_ARRAY_SUBSET_NOP.to_vec());
}

#[no_mangle]
extern "C" fn patch_load_value_from_ini_call() {
    debug_dll::debug_logger(&format!("load_value_from_ini {:p}", load_value_from_ini as *const ()));
    debug_dll::patch_calls(debug_dll::LOAD_VALUE_FROM_INI_ADDRESS_ARRAY.to_vec(), load_value_from_ini as u32);
}

#[no_mangle]
extern "cdecl" fn load_int_from_ini(section_address: &u32, header_address: &u32, default: i32) -> u32 {
    debug_dll::debug_logger(&format!(
        "load_int_from_ini {:p} {:p} default: {}",
        *section_address as *const (), *header_address as *const (), default
    ));
    let section = debug_dll::get_string_from_memory(*section_address);
    let header = debug_dll::get_string_from_memory(*header_address);
    let mut zoo_ini = Ini::new();
    zoo_ini.load(get_ini_path()).unwrap();
    let result = load_ini::load_int_with_default(&zoo_ini, &section, &header, default) as u32;
    debug_dll::debug_logger(&format!("load_int_from_ini {} {} result: {}", section, header, result));
    result
}

#[no_mangle]
extern "cdecl" fn load_value_from_ini<'a>(result_address: &'a u32, section_address: &u32, header_address: &u32, default_address: &u32) -> &'a u32 {
    debug_dll::debug_logger(&format!(
        "load_value_from_ini {:p} {:p} default: {:p}",
        *section_address as *const (), *header_address as *const (), *default_address as *const ()
    ));
    let section = debug_dll::get_string_from_memory(*section_address);
    let header = debug_dll::get_string_from_memory(*header_address);
    let default = debug_dll::get_string_from_memory(*default_address);
    let mut zoo_ini = Ini::new();
    zoo_ini.load(get_ini_path()).unwrap();
    let result = load_ini::load_string_with_default(&zoo_ini, &section, &header, &default);

    debug_dll::debug_logger(&format!("load_value_from_ini {} {} result: {}", section, header, result));
    debug_dll::debug_logger(&format!("encoding string at address: {:p}", *result_address as *const ()));
    debug_dll::save_string_to_memory(*result_address, &result);
    result_address
}

fn get_ini_path() -> String {
    let mut base_path = debug_dll::get_base_path();
    base_path.push("zoo.ini");
    base_path.to_str().unwrap().to_string()
}

#[hook_module("zoo.exe")]
mod zoo_misc {
    #[hook(unsafe extern "thiscall" UIControl_useAnimation, offset = 0x0000b1f89)]
    fn ui_control_use_animation(this_ptr: u32, param_1: u32, param_2: bool) {
        unsafe { UIControl_useAnimation.call(this_ptr, param_1, param_2) }
    }

    #[hook(unsafe extern "thiscall" UIControl_setAnimation, offset = 0x0000b1aa0)]
    fn ui_control_set_animation(this_ptr: u32, param_1: u32, param_2: bool) {
        // if param_1 == 0 {
        //     info!("UIControl::setAnimation {:#x} {:#x} {}", this_ptr, param_1, param_2);
        // } else {
        //     let param_1_string = get_string_from_memory(param_1);
        //     if param_1_string.starts_with("openzt") || param_1_string.starts_with("ui/infoimg") {
        //         info!(
        //             "UIControl::setAnimation {:#x} {:#x} ({}) {}",
        //             this_ptr, param_1, param_1_string, param_2
        //         );
        //     }
        // }
        unsafe { UIControl_setAnimation.call(this_ptr, param_1, param_2) }
    }

    // 0x0000176ce
    #[hook(unsafe extern "cdecl" UIControl_UILoadAnimation, offset = 0x0000176ce)]
    fn ui_control_ui_load_animation(param_1: u32, param_2: u32, param_3: u32) -> u8 {
        unsafe { UIControl_UILoadAnimation.call(param_1, param_2, param_3) }
    }

    #[hook(unsafe extern "thiscall" BFAnimCache_findAnim, offset = 0x000001fdd)]
    fn bf_anim_cache_find_anim(this_ptr: u32, param_1: u32, param_2: u32) -> u32 {
        unsafe { BFAnimCache_findAnim.call(this_ptr, param_1, param_2) }
    }

    #[hook(unsafe extern "thiscall" GXLLEAnimSet_attempt_bfconfigfile, offset = 0x0000b967)]
    fn zoo_gxlleanimset_attempt_bfconfigfile(this_ptr: u32, param_1: u32, param_2: u32) -> u8 {
        unsafe { GXLLEAnimSet_attempt_bfconfigfile.call(this_ptr, param_1, param_2) }
    }

    #[hook(unsafe extern "thiscall" GXLLEAnim_attempt, offset = 0x000011e21)]
    fn zoo_gxlleanim_attempt(this_ptr: u32, param_1: u32) -> u8 {
        unsafe { GXLLEAnim_attempt.call(this_ptr, param_1) }
    }

    #[hook(unsafe extern "thiscall" OOAnalyzer_GXLLEAnim_prepare, offset = 0x0000bbc1)]
    fn zoo_ooanalyzer_gxlleanim_prepare(this_ptr: u32, param_1: u32) -> u8 {
        unsafe { OOAnalyzer_GXLLEAnim_prepare.call(this_ptr, param_1) }
    }

    #[hook(unsafe extern "thiscall" BFConfigFile_attempt, offset = 0x00009ac0)]
    fn zoo_bfconfigfile_attempt(this_ptr: u32, param_1: u32) -> u8 {
        unsafe { BFConfigFile_attempt.call(this_ptr, param_1) }
    }
}
