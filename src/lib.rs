#![feature(abi_thiscall)]

use configparser::ini::Ini;

use std::net::TcpStream;

use std::sync::Mutex;

use tracing::{info, Level};

use retour_utils::hook_module;

mod bfregistry;


#[cfg(target_os = "windows")]
use winapi::um::winnt::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH, DLL_THREAD_ATTACH, DLL_THREAD_DETACH};

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

    let stream = TcpStream::connect("127.0.0.1:1492").unwrap();

    let subscriber = tracing_subscriber::fmt()
        .with_writer(Mutex::new(stream))
        .with_max_level(Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber).unwrap_or_else(|error| {
        panic!("Failed to init tracing: {error}")
    });

    info!("openzt.dll Loaded");
}


#[hook_module("zoo.exe")]
mod zoo_ini {
    use tracing::{info};

    use crate::debug_dll::{get_string_from_memory, get_from_memory};

    use crate::load_debug_settings_from_ini;

    use crate::cls_0x40190a;

    #[hook(unsafe extern "cdecl" LoadDebugSettingsFromIniHook, offset = 0x00179f4c)]
    fn load_debug_settings_from_ini_detour() {
        info!("Detour via macro (Debug Ini Settings)");
        // unsafe { LoadDebugSettingsFromIniHook.call() }
        load_debug_settings_from_ini();
    }

    #[hook(unsafe extern "cdecl" LoadIniValueMaybeHook, offset = 0x0001b4bc)]
    // fn load_ini_value_log_detour(class_ptr: u32, ini_section: LPCSTR, ini_key: LPCSTR, ini_default: LPCSTR) -> u32 {
    fn load_ini_value_log_detour(class_ptr: u32, ini_section: u32, ini_key: u32, ini_default: u32) -> u32 {
        info!("Detour via macro (LoadIniValueMaybeHook)");
        let deref_class_ptr = get_from_memory::<u32>(class_ptr);
        info!("class_ptr: {:#08x}", class_ptr);
        info!("deref_class_ptr: {:#08x}", deref_class_ptr);

        let cls_0x40190a_addr: *const cls_0x40190a = class_ptr as *const _;
        let cls_0x40190a_obj = unsafe { &*cls_0x40190a_addr };
        info!("cls_0x40190a_obj: {:#?}", cls_0x40190a_obj);

        info!("ini_section: {}", get_string_from_memory(get_from_memory::<u32>(ini_section)));
        info!("ini_key: {}", get_string_from_memory(get_from_memory::<u32>(ini_key)));
        info!("ini_default: {}", get_string_from_memory(get_from_memory::<u32>(ini_default)));
        let return_value = unsafe { LoadIniValueMaybeHook.call(class_ptr, ini_section, ini_key, ini_default) };
        info!("return_value: {}", get_string_from_memory(get_from_memory::<u32>(return_value)));

        let cls_0x40190a_addr: *const cls_0x40190a = class_ptr as *const _;
        let cls_0x40190a_obj = unsafe { &*cls_0x40190a_addr };
        info!("cls_0x40190a_obj: {:#?}", cls_0x40190a_obj);

        return_value
    }
}

#[hook_module("zoo.exe")]
mod zoo_bf_registry {
    use crate::debug_dll::{get_string_from_memory, get_from_memory};

    use crate::bfregistry::{get_from_registry, add_to_registry};

    #[hook(unsafe extern "thiscall" BFRegistry_prtGetHook, offset = 0x000bdd22)]
    fn prt_get(_this_prt: u32, class_name: u32, _delimeter_maybe: u8) -> u32 {
        get_from_registry(get_string_from_memory(get_from_memory::<u32>(class_name))).unwrap()
    }

    #[hook(unsafe extern "cdecl" BFRegistry_AddHook, offset = 0x001770e5)]
    fn add_to_bfregistry(param_1: u32, param_2: u32) -> u32 {
        let param_1_string = get_string_from_memory(get_from_memory::<u32>(param_1));
        add_to_registry(&param_1_string, param_2);
        0x638001
    }

    #[hook(unsafe extern "cdecl" BFRegistry_AddUIHook, offset = 0x001774bf)]
    fn add_to_bfregistry_ui(param_1: u32, param_2: u32) -> u32  {
        let param_1_string = get_string_from_memory(get_from_memory::<u32>(param_1));
        add_to_registry(&param_1_string, param_2);
        0x638001
    }

}


#[no_mangle]
extern "system" fn DllMain(module: u8, reason: u32, _reserved: u8) -> i32 {
    match reason {
        DLL_PROCESS_ATTACH => {
            dll_first_load();
            info!("DllMain: DLL_PROCESS_ATTACH: {}, {} {}", module, reason, _reserved);

            unsafe { 
                if cfg!(feature = "ini") {
                    zoo_ini::init_detours().unwrap();
                }
                if cfg!(feature = "bf_registry") {
                    zoo_bf_registry::init_detours().unwrap();
                }
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
    debug_dll::debug_logger(&format!("load_int_from_ini {:p} {:p} default: {}", *section_address as *const (), *header_address as *const (), default));
    let section = debug_dll::get_string_from_memory(*section_address);
    let header = debug_dll::get_string_from_memory(*header_address);
    let mut zoo_ini = Ini::new();
    zoo_ini.load(get_ini_path()).unwrap();
    let result = load_ini::load_int_with_default(&zoo_ini, &section, &header, default) as u32;
    debug_dll::debug_logger(&format!("load_int_from_ini {} {} result: {}", section, header, result));
    return result;
}

#[no_mangle]
extern "cdecl" fn load_value_from_ini<'a>(result_address: &'a u32, section_address: &u32, header_address: &u32, default_address: &u32) -> &'a u32{
    debug_dll::debug_logger(&format!("load_value_from_ini {:p} {:p} default: {:p}", *section_address as *const (), *header_address as *const (), *default_address as *const ()));
    let section = debug_dll::get_string_from_memory(*section_address);
    let header = debug_dll::get_string_from_memory(*header_address);
    let default = debug_dll::get_string_from_memory(*default_address);
    let mut zoo_ini = Ini::new();
    zoo_ini.load(get_ini_path()).unwrap();
    let result = load_ini::load_string_with_default(&zoo_ini, &section, &header, &default);

    debug_dll::debug_logger(&format!("load_value_from_ini {} {} result: {}", section, header, result));
    debug_dll::debug_logger(&format!("encoding string at address: {:p}", *result_address as *const ()));
    debug_dll::save_string_to_memory(*(&result_address as &u32), &result);
    // ptr::write(result_address as *mut _, result);
    return result_address;
}

fn get_ini_path() -> String {
    let mut base_path = debug_dll::get_base_path();
    base_path.push("zoo.ini");
    base_path.to_str().unwrap().to_string()
}