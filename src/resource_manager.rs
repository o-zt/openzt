use core::slice;
use core::fmt::Display;
use std::{
    collections::{HashMap, HashSet}, ffi::CString, fmt, fs::File, io::{self, BufReader, Read}, iter::Zip, path::{Path, PathBuf}, sync::Mutex
};


use bf_configparser::ini::{Ini, WriteOptions};
use once_cell::sync::Lazy;
use retour_utils::hook_module;
use tracing::{error, info};
use walkdir::WalkDir;
use zip::read::ZipFile;

use crate::{
    animation::Animation,
    console::{add_to_command_register, CommandError},
    debug_dll::{get_from_memory, get_string_from_memory, save_to_memory},
    mods,
};

const GLOBAL_BFRESOURCEMGR_ADDRESS: u32 = 0x006380C0;

#[derive(Debug, Clone)]
pub enum ZTFile {
    Text(CString, ZTFileType, u32),
    RawBytes(Box<[u8]>, ZTFileType, u32),
}

#[derive(Debug, Clone)]
pub enum ZTFileType {
    Ai,
    Ani,
    Cfg,
    Lyt,
    Scn,
    Uca,
    Ucs,
    Ucb,
    Ini,
    Txt,
    Toml,
    Animation,
    Palette,
    TGA,
    Wav,
    Lle,
    Bmp,
}

impl From<BFResourcePtr> for ZTFile {
    fn from(bf_resource_ptr: BFResourcePtr) -> Self {
        let filename = get_string_from_memory(bf_resource_ptr.bf_resource_name_ptr);
        let file_extension = Path::new(&filename)
            .extension()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default();
        let file_size = bf_resource_ptr.content_size;
        let data = bf_resource_ptr.data_ptr;
        match file_extension {
            "ai" => ZTFile::Text(
                unsafe { CString::from_raw(data as *mut i8) },
                ZTFileType::Ai,
                file_size,
            ),
            "cfg" => ZTFile::Text(
                unsafe { CString::from_raw(data as *mut i8) },
                ZTFileType::Cfg,
                file_size,
            ),
            "lyt" => ZTFile::Text(
                unsafe { CString::from_raw(data as *mut i8) },
                ZTFileType::Lyt,
                file_size,
            ),
            "scn" => ZTFile::Text(
                unsafe { CString::from_raw(data as *mut i8) },
                ZTFileType::Scn,
                file_size,
            ),
            "uca" => ZTFile::Text(
                unsafe { CString::from_raw(data as *mut i8) },
                ZTFileType::Uca,
                file_size,
            ),
            "ucs" => ZTFile::Text(
                unsafe { CString::from_raw(data as *mut i8) },
                ZTFileType::Ucs,
                file_size,
            ),
            "ucb" => ZTFile::Text(
                unsafe { CString::from_raw(data as *mut i8) },
                ZTFileType::Ucb,
                file_size,
            ),
            "ani" => ZTFile::Text(
                unsafe { CString::from_raw(data as *mut i8) },
                ZTFileType::Ani,
                file_size,
            ),
            "ini" => ZTFile::Text(
                unsafe { CString::from_raw(data as *mut i8) },
                ZTFileType::Ini,
                file_size,
            ),
            "txt" => ZTFile::Text(
                unsafe { CString::from_raw(data as *mut i8) },
                ZTFileType::Txt,
                file_size,
            ),
            "tga" => ZTFile::RawBytes(
                unsafe {
                    Box::from_raw(slice::from_raw_parts_mut(
                        data as *mut _,
                        file_size as usize,
                    ))
                },
                ZTFileType::TGA,
                file_size,
            ),
            "pal" => ZTFile::RawBytes(
                unsafe {
                    Box::from_raw(slice::from_raw_parts_mut(
                        data as *mut _,
                        file_size as usize,
                    ))
                },
                ZTFileType::Palette,
                file_size,
            ),
            "wav" => ZTFile::RawBytes(
                unsafe {
                    Box::from_raw(slice::from_raw_parts_mut(
                        data as *mut _,
                        file_size as usize,
                    ))
                },
                ZTFileType::Wav,
                file_size,
            ),
            "lle" => ZTFile::RawBytes(
                unsafe {
                    Box::from_raw(slice::from_raw_parts_mut(
                        data as *mut _,
                        file_size as usize,
                    ))
                },
                ZTFileType::Lle,
                file_size,
            ),
            "bmp" => ZTFile::RawBytes(
                unsafe {
                    Box::from_raw(slice::from_raw_parts_mut(
                        data as *mut _,
                        file_size as usize,
                    ))
                },
                ZTFileType::Bmp,
                file_size,
            ),
            _ => ZTFile::RawBytes(
                unsafe {
                    Box::from_raw(slice::from_raw_parts_mut(
                        data as *mut _,
                        file_size as usize,
                    ))
                },
                ZTFileType::Animation,
                file_size,
            ),
        }
    }
}

impl ZTFile {
    pub fn new_text(
        file_name: String,
        file_size: u32,
        data: CString,
    ) -> Result<ZTFile, &'static str> {
        let file_extension = Path::new(&file_name)
            .extension()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default();
        match file_extension {
            "ai" => Ok(ZTFile::Text(data, ZTFileType::Ai, file_size)),
            "cfg" => Ok(ZTFile::Text(data, ZTFileType::Cfg, file_size)),
            "lyt" => Ok(ZTFile::Text(data, ZTFileType::Lyt, file_size)),
            "scn" => Ok(ZTFile::Text(data, ZTFileType::Scn, file_size)),
            "uca" => Ok(ZTFile::Text(data, ZTFileType::Uca, file_size)),
            "ucs" => Ok(ZTFile::Text(data, ZTFileType::Ucs, file_size)),
            "ucb" => Ok(ZTFile::Text(data, ZTFileType::Ucb, file_size)),
            "ani" => Ok(ZTFile::Text(data, ZTFileType::Ani, file_size)),
            "ini" => Ok(ZTFile::Text(data, ZTFileType::Ini, file_size)),
            "txt" => Ok(ZTFile::Text(data, ZTFileType::Txt, file_size)),
            "toml" => Ok(ZTFile::Text(data, ZTFileType::Toml, file_size)),
            _ => Err("Invalid file type"),
        }
    }

    pub fn new_raw_bytes(file_name: String, file_size: u32, data: Box<[u8]>) -> ZTFile {
        let file_extension = Path::new(&file_name)
            .extension()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default();
        match file_extension {
            "tga" => ZTFile::RawBytes(data, ZTFileType::TGA, file_size),
            "pal" => ZTFile::RawBytes(data, ZTFileType::Palette, file_size),
            "wav" => ZTFile::RawBytes(data, ZTFileType::Wav, file_size),
            "lle" => ZTFile::RawBytes(data, ZTFileType::Lle, file_size),
            "bmp" => ZTFile::RawBytes(data, ZTFileType::Bmp, file_size),
            _ => ZTFile::RawBytes(data, ZTFileType::Animation, file_size),
        }
    }
}

pub trait FromZipFile<T> {
    fn from_zip_file(file: &mut ZipFile) -> io::Result<T>;
}

// impl FromZipFile<String> for String {
//     fn from_zip_file(file: &mut ZipFile) -> io::Result<String> {
//         let mut buffer = vec![0; file.size() as usize];
//         file.read(&mut buffer[..])?;
//         Ok(String::from_utf8_lossy(&buffer[..]).to_string())
//     }
// }

// impl FromZipFile<Vec<u8>> for Vec<u8> {
//     fn from_zip_file(file: &mut ZipFile) -> io::Result<Self> {
//         let mut buffer = vec![0; file.size() as usize];
//         file.read(&mut buffer[..])?;
//         Ok(buffer)
//     }
// }

// impl FromZipFile<CString> for CString {
//     fn from_zip_file(file: &mut ZipFile) -> io::Result<Self> {
//         let mut buffer = vec![0; file.size() as usize];
//         file.read(&mut buffer[..])?;
//         Ok(CString::new(
//             String::from_utf8_lossy(&buffer[..]).to_string(),
//         )?)
//     }
// }

// fn load_open_zt_mod(entry: &Path, file: &mut ZipFile) {
//     info!("Loading OpenZT mod: {} file: {}", file.name(), entry.display());
//     // add_to_openzt_mod_buffer(entry, file.clone());
// }

fn add_file_to_maps(entry: &Path, file: &mut ZipFile) {
    let lowercase_file_name = file.name().to_lowercase();
    if check_file(&lowercase_file_name) {
        // File already exists, skip loading
        return;
    }
    // TODO: Figure out issues with loading ini, txt and non-text files
    // NOTE: Non-text files seem to work fine when not using mods
    let file_extension = Path::new(&lowercase_file_name)
        .extension()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default();
    if matches!(
        file_extension,
        "ai" | "ani" | "cfg" | "lyt" | "scn" | "uca" | "ucs" | "ucb") {
        // | "ini" | "txt") {
        add_txt_file_to_map(entry, file);
        // } else if matches!(file_extension, "tga" | "pal" | "wav" | "lle" | "bmp" | "") {
        // add_raw_bytes_file_to_map(entry, file);
    }
}

pub fn add_txt_file_to_map_with_path_override(entry: &Path, file: &mut ZipFile, path: String) {
    let mut buffer = vec![0; file.size() as usize].into_boxed_slice();
    match file.read_exact(&mut buffer) {
        Ok(bytes_read) => bytes_read,
        Err(e) => {
            error!(
                "Error reading file: {} {} -> {}",
                entry.display(),
                file.name(),
                e
            );
            return;
        }
    };

    let intermediate_string = String::from_utf8_lossy(&buffer).to_string();

    let file_size = intermediate_string.len();
    let file_contents = match CString::new(intermediate_string) {
        Ok(c_string) => c_string,
        Err(e) => {
            error!(
                "Error converting file contents to CString: {} {} -> {}",
                entry.display(),
                file.name(),
                e
            );
            return;
        }
    };

    let ztfile = match ZTFile::new_text(path.clone(), file_size as u32, file_contents) {
        Ok(ztfile) => ztfile,
        Err(e) => {
            error!(
                "Error creating ZTFile from text: {} {} -> {}",
                entry.display(),
                file.name(),
                e
            );
            return;
        }
    };

    add_ztfile(
        entry,
        path,
        ztfile,
    );
}

pub fn add_txt_file_to_map(entry: &Path, file: &mut ZipFile) {
    let file_name = file.name().to_string().to_lowercase();

    add_txt_file_to_map_with_path_override(entry, file, file_name)
}

pub fn add_raw_bytes_to_map_with_path_override(entry: &Path, file: &mut ZipFile, path: String) {
    let mut buffer = vec![0; file.size() as usize].into_boxed_slice();
    match file.read_exact(&mut buffer) {
        Ok(_) => {}
        Err(e) => {
            error!(
                "Error reading file: {} {} -> {}",
                entry.display(),
                file.name(),
                e
            );
            return;
        }
    };

    let file_size = file.size() as u32;
    add_ztfile(
        entry,
        path.clone(),
        ZTFile::new_raw_bytes(path, file_size, buffer),
    );
}

pub fn add_raw_bytes_file_to_map(entry: &Path, file: &mut ZipFile) {
    let file_name = file.name().to_string().to_lowercase();
    add_raw_bytes_to_map_with_path_override(entry, file, file_name)
}

// TODO: Add temp mod structure which holds resources based on full zip path. Once fully loaded, we can move to main resource map with the mod_id as a prefix
// 

// ModZipPath -> FilePath -> FilePtr
// TODO: This should be local to the 'handle_ztd' function
// static OPENZT_MOD_BUFFER_MAP: Lazy<Mutex<HashMap<String, HashMap<String, ZipFile>>>> = // What to do here
//     Lazy::new(|| Mutex::new(HashMap::new()));

// fn clear_openzt_mod_buffer() {
//     let Ok(mut binding) = OPENZT_MOD_BUFFER_MAP.lock() else {
//         error!("Failed to lock openzt mod buffer map; returning from delete_openzt_mod_buffer for {}", mod_id);
//         return;
//     };
//     binding.clear();
// }

// // TODO: Check this actually works
// fn add_to_openzt_mod_buffer(file_path: &Path, file: ZipFile) {
//     let mod_zip_path = file_path.to_str().unwrap_or(default).to_string();
//     if mod_zip_path == "" {
//         error!("Unable to read mod zip path: {}", file_path.display());
//         return;
//     }
//     let Ok(mut binding) = OPENZT_MOD_BUFFER_MAP.lock() else {
//         error!("Failed to lock openzt mod buffer map; returning from add_to_openzt_mod_buffer for {}", mod_zip_path);
//         return;
//     };
//     if binding.entry(mod_zip_path.clone()).or_insert(HashMap::new()).insert(file.name().to_string(), file).is_some() {
//         error!("File already exists in openzt mod buffer: {}", file.name());
//     }
// }

// fn list_openzt_mod_buffer() {
//     let Ok(binding) = OPENZT_MOD_BUFFER_MAP.lock() else {
//         error!("Failed to lock openzt mod buffer map; returning from list_openzt_mod_buffer");
//         return;
//     };
//     for (mod_zip_path, files) in binding.iter() {
//         info!("Mod: {}", mod_zip_path);
//         for (file_path, _) in files.iter() {
//             info!("File: {}", file_path);
//         }
//     }
// }

// Contains a mapping of file_paths to BFResourcePtrs
static RESOURCE_STRING_TO_PTR_MAP: Lazy<Mutex<HashMap<String, u32>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

static RESOURCE_PTR_PTR_SET: Lazy<Mutex<HashSet<u32>>> = Lazy::new(|| Mutex::new(HashSet::new()));

pub fn add_ptr_ptr(ptr_ptr: u32) {
    let Ok(mut binding) = RESOURCE_PTR_PTR_SET.lock() else {
        error!("Failed to lock resource ptr ptr set; returning from add_ptr_ptr for {}", ptr_ptr);
        return;
    };
    binding.insert(ptr_ptr);
}

pub fn check_ptr_ptr(ptr_ptr: u32) -> bool {
    let Ok(binding) = RESOURCE_PTR_PTR_SET.lock() else {
        error!("Failed to lock resource ptr ptr set; returning false from check_ptr_ptr for {}", ptr_ptr);
        return false;
    };
    binding.contains(&ptr_ptr)
}

pub fn check_file(file_name: &str) -> bool {
    let Ok(binding) = RESOURCE_STRING_TO_PTR_MAP.lock() else {
        error!("Failed to lock resource string to ptr map; returning false from check_file for {}", file_name);
        return false;
    };
    binding.contains_key(&file_name.to_lowercase())
}

pub fn get_file_ptr(file_name: &str) -> Option<u32> {
    let Ok(binding) = RESOURCE_STRING_TO_PTR_MAP.lock() else {
        error!("Failed to lock resource string to ptr map; returning None from get_file_ptr for {}", file_name);
        return None;
    };
    binding.get(&file_name.to_lowercase()).copied()
}

fn get_num_resources() -> usize {
    let Ok(binding) = RESOURCE_STRING_TO_PTR_MAP.lock() else {
        error!("Failed to lock resource string to ptr map; returning 0 from get_num_resources");
        return 0;
    };
    binding.len()
}

fn command_list_resource_strings(_args: Vec<&str>) -> Result<String, CommandError> {
    let Ok(binding) = RESOURCE_STRING_TO_PTR_MAP.lock() else {
        error!("Failed to lock resource string to ptr map; returning from command_list_resource_strings");
        return Err(CommandError::new("Failed to lock resource string to ptr map".to_string()));
    };
    let mut result_string = String::new();
    for (resource_string, _) in binding.iter() {
        result_string.push_str(&format!("{}\n", resource_string));
    }
    Ok(result_string)
}

fn add_ztfile(path: &Path, file_name: String, ztfile: ZTFile) {
    let Some(ztd_path) = path.to_str() else {
        error!("Failed to convert path to string: {}", path.display());
        return;
    };
    let mut ztd_path = ztd_path.to_string();
    ztd_path = ztd_path.replace("./", "zip::./").replace('\\', "/");
    let lowercase_filename = file_name.to_lowercase();

    let Ok(mut binding) = RESOURCE_STRING_TO_PTR_MAP.lock() else {
        error!("Failed to lock resource string to ptr map; returning from add_ztfile for {}", file_name);
        return;
    };

    let bf_zip_name_ptr = match CString::new(ztd_path.clone()) {
        Ok(c_string) => c_string.into_raw() as u32,
        Err(e) => {
            error!("Error converting zip name to CString: {} -> {}", ztd_path, e);
            return;
        }
    };
    let bf_resource_name_ptr  = match CString::new(lowercase_filename.clone()) {
        Ok(c_string) => c_string.into_raw() as u32,
        Err(e) => {
            error!("Error converting resource name to CString: {} -> {}", lowercase_filename, e);
            return;
        }
    };

    match ztfile {
        ZTFile::Text(data, _, length) => {
            let ptr = data.into_raw() as u32;
            let resource_ptr = Box::into_raw(Box::new(BFResourcePtr {
                num_refs: 100, // We set this very high to prevent the game from unloading the resource
                bf_zip_name_ptr,
                bf_resource_name_ptr,
                data_ptr: ptr,
                content_size: length,
            }));

            binding.insert(file_name.clone(), resource_ptr as u32);
        }
        ZTFile::RawBytes(data, _, length) => {
            let ptr = data.as_ptr() as u32;
            std::mem::forget(data);
            let resource_ptr = Box::into_raw(Box::new(BFResourcePtr {
                num_refs: 100, // We set this very high to prevent the game from unloading the resource
                bf_zip_name_ptr,
                bf_resource_name_ptr,
                data_ptr: ptr,
                content_size: length,
            }));

            binding.insert(lowercase_filename.clone(), resource_ptr as u32);
        }
    }
}

pub fn modify_ztfile<F>(file_name: &str, modifier: F) -> Result<(), &'static str>
where
    F: Fn(&mut BFResourcePtr),
{
    let Some(bf_resource_ptr_ptr) = get_file_ptr(file_name) else {
        info!("File not found: {}", file_name);
        return Err("File not found");
    };
    let mut bf_resource_ptr = get_from_memory::<BFResourcePtr>(bf_resource_ptr_ptr);

    modifier(&mut bf_resource_ptr);

    save_to_memory::<BFResourcePtr>(bf_resource_ptr_ptr, bf_resource_ptr.clone());

    Ok(())
}

pub fn modify_ztfile_as_ini<F>(file_name: &str, modifier: F) -> Result<(), &'static str>
where
    F: Fn(&mut Ini),
{
    modify_ztfile(file_name, |file: &mut BFResourcePtr| {
        let c_string = unsafe { CString::from_raw(file.data_ptr as *mut i8) };
        let c_string_as_string = c_string.to_string_lossy().to_string();
        let mut cfg = Ini::new_cs();
        cfg.set_comment_symbols(&[';', '#', ':']);
        if let Err(err) = cfg.read(c_string_as_string) {
            error!("Error reading ini: {}", err);
            return;
        };

        modifier(&mut cfg);

        let mut write_options = WriteOptions::default();
        write_options.space_around_delimiters = true;
        write_options.blank_lines_between_sections = 1;
        let new_string = cfg.pretty_writes(&write_options);
        file.content_size = new_string.len() as u32;

        let Ok(new_c_string) = CString::new(new_string) else {
            error!("Error converting ini to CString after modifying {} writing unchanged version", file_name);
            return;
        };
        file.data_ptr = new_c_string.into_raw() as u32;
    })
}

pub fn modify_ztfile_as_animation<F>(file_name: &str, modifier: F) -> Result<(), &'static str>
where
    F: Fn(&mut Animation),
{
    modify_ztfile(file_name, |file: &mut BFResourcePtr| {
        info!("Modifying animation");
        let data_vec: Box<[u8]> = unsafe {
            Box::from_raw(slice::from_raw_parts_mut(
                file.data_ptr as *mut _,
                file.content_size as usize,
            ))
        };
        let mut animation = Animation::parse(&data_vec);
        modifier(&mut animation);
        let (new_animation_bytes, length) = animation.write();
        let boxed_slice = new_animation_bytes.into_boxed_slice();
        let data_ptr = boxed_slice.as_ptr() as u32;
        std::mem::forget(boxed_slice);
        file.data_ptr = data_ptr;
        file.content_size = length as u32;
    })
}

#[derive(Debug)]
#[repr(C)]
struct BFResourceMgr {
    resource_array_start: u32,
    resource_array_end: u32,
    resource_array_buffer_end: u32,
    unknown_u32_1: u32,
    unknown_u32_2: u32,
    unknown_u8_1: u8,
}

#[derive(Debug)]
#[repr(C)]
struct BFResourceDir {
    class: u32,
    unknown_u32_1: u32,
    dir_name_string_start: u32,
    dir_name_string_end: u32,
    unknown_u32_2: u32,
    num_child_files: u32,
    unknown_u32_3: u32,
}

#[derive(Debug)]
#[repr(C)]
struct BFResourceZip {
    class: u32,
    unknown_u32_1: u32,
    unknown_u32_2: u32,
    unknown_u32_3: u32,
    zip_name_string_start: u32,
    contents_tree: u32, //? contents end?
}

#[derive(Debug)]
#[repr(C)]
struct BFResourceDirContents {
    dir: BFResourceDir,
    zips: Vec<BFResourceZip>,
}

#[derive(Debug)]
#[repr(C)]
struct BFResource {
    bf_resource_ptr_ptr: u32,
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct BFResourcePtr {
    pub num_refs: u32,
    pub bf_zip_name_ptr: u32,
    pub bf_resource_name_ptr: u32,
    pub data_ptr: u32,
    pub content_size: u32,
}

#[derive(Debug)]
#[repr(C)]
struct GXLLEAnim {
    padding: [u8; 5],
    bfresource_maybe: u32,
}

impl fmt::Display for BFResourcePtr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BFResourcePtr {{ num_refs: {:#x}, bf_zip_name: {}, bf_resource_name: {}, data_ptr: {:#x}, content_size: {:#x} }}", self.num_refs, get_string_from_memory(self.bf_zip_name_ptr), get_string_from_memory(self.bf_resource_name_ptr), self.data_ptr, self.content_size)
    }
}

trait Name {
    fn name(&self) -> String;
}

impl Name for BFResourceDir {
    fn name(&self) -> String {
        get_string_from_memory(self.dir_name_string_start)
    }
}

impl Name for BFResourceZip {
    fn name(&self) -> String {
        get_string_from_memory(self.zip_name_string_start)
    }
}

fn read_bf_resource_mgr_from_memory() -> BFResourceMgr {
    get_from_memory::<BFResourceMgr>(GLOBAL_BFRESOURCEMGR_ADDRESS)
}

fn read_bf_resource_dir_contents_from_memory() -> Vec<BFResourceDirContents> {
    info!("Reading BFResourceDir from memory");
    let bf_resource_mgr = read_bf_resource_mgr_from_memory();
    let mut bf_resource_dir_contents: Vec<BFResourceDirContents> = Vec::new();
    let mut bf_resource_dir_ptr = bf_resource_mgr.resource_array_start;
    let mut bf_resource_zips: Vec<BFResourceZip> = Vec::new();
    let mut current_bf_resource_dir =
        get_from_memory::<BFResourceDir>(get_from_memory::<u32>(bf_resource_dir_ptr));
    bf_resource_dir_ptr += 4;

    while bf_resource_dir_ptr < bf_resource_mgr.resource_array_end {
        let class = get_from_memory::<u32>(get_from_memory::<u32>(bf_resource_dir_ptr));
        match class {
            0x630aec => {
                bf_resource_dir_contents.push(BFResourceDirContents {
                    dir: current_bf_resource_dir,
                    zips: bf_resource_zips,
                });
                current_bf_resource_dir =
                    get_from_memory::<BFResourceDir>(get_from_memory::<u32>(bf_resource_dir_ptr));
                bf_resource_zips = Vec::new();
                bf_resource_dir_ptr += 4;
            }
            0x630b0c => {
                bf_resource_zips.push(get_from_memory::<BFResourceZip>(get_from_memory::<u32>(
                    bf_resource_dir_ptr,
                )));
                bf_resource_dir_ptr += 4;
            }
            _ => {
                error!("Unknown class: 0x{:X}", class);
                bf_resource_dir_ptr += 4;
            }
        }
    }
    bf_resource_dir_contents.push(BFResourceDirContents {
        dir: current_bf_resource_dir,
        zips: bf_resource_zips,
    });
    bf_resource_dir_contents
}

impl fmt::Display for BFResourceMgr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BFResourceMgr {{ resource_array_start: 0x{:X}, resource_array_end: 0x{:X}, resource_array_buffer_end: 0x{:X}, unknown_u32_1: 0x{:X}, unknown_u32_2: 0x{:X}, unknown_u8_1: 0x{:X} }}", self.resource_array_start, self.resource_array_end, self.resource_array_buffer_end, self.unknown_u32_1, self.unknown_u32_2, self.unknown_u8_1)
    }
}

impl fmt::Display for BFResourceDir {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let dir_name_string = get_string_from_memory(self.dir_name_string_start);
        write!(f, "BFResourceDir {{ class: 0x{:X}, unknown_u32_1: 0x{:X}, dir_name: {}, num_bfr_zip: 0x{:X}, unknown_u32_2: 0x{:X} }}", self.class, self.unknown_u32_1, dir_name_string, self.num_child_files, self.unknown_u32_2)
    }
}

impl fmt::Display for BFResourceZip {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let zip_name_string = get_string_from_memory(self.zip_name_string_start);
        write!(f, "BFResourceZip {{ class: 0x{:X}, unknown_u32_1: 0x{:X}, unknown_u32_2: 0x{:X}, unknown_u32_3: 0x{:X}, zip_name: {}, contents_tree: 0x{:X} }}", self.class, self.unknown_u32_1, self.unknown_u32_2, self.unknown_u32_3, zip_name_string, self.contents_tree)
    }
}

fn command_list_resources(_args: Vec<&str>) -> Result<String, CommandError> {
    let mut result_string = String::new();
    let bf_resource_dir_contents = read_bf_resource_dir_contents_from_memory();
    for bf_resource_dir_content in bf_resource_dir_contents {
        let bf_resource_dir = bf_resource_dir_content.dir;
        result_string.push_str(&format!(
            "{} ({})\n",
            get_string_from_memory(bf_resource_dir.dir_name_string_start),
            bf_resource_dir.num_child_files
        ));
        let bf_resource_zips = bf_resource_dir_content.zips;
        for bf_resource_zip in bf_resource_zips {
            result_string.push_str(&format!(
                "{}\n",
                get_string_from_memory(bf_resource_zip.zip_name_string_start)
            ));
        }
    }
    Ok(result_string)
}

fn command_get_bf_resource_mgr(_args: Vec<&str>) -> Result<String, CommandError> {
    let bf_resource_mgr = read_bf_resource_mgr_from_memory();
    Ok(format!("{}", bf_resource_mgr))
}

pub fn init() {
    add_to_command_register("list_resources".to_owned(), command_list_resources);
    add_to_command_register("get_bfresourcemgr".to_owned(), command_get_bf_resource_mgr);
    if unsafe { zoo_resource_mgr::init_detours() }.is_err() {
        error!("Failed to init resource_mgr detours");
    };
    add_to_command_register("list_resource_strings".to_string(), command_list_resource_strings);
    add_handler(Handler::new(None, None, add_file_to_maps, ModType::Legacy));
    // add_handler(Handler::new(None, None, load_open_zt_mod, ModType::OpenZT))
    // TODO: Add OpenZT mod handler
}

#[hook_module("zoo.exe")]
pub mod zoo_resource_mgr {
    use bf_configparser::ini::Ini; //TODO: Replace with custom ini parser
    use tracing::info;

    use super::{check_file, get_file_ptr, load_resources, BFResourcePtr};
    use crate::debug_dll::{get_ini_path, get_string_from_memory, save_to_memory};

    #[hook(unsafe extern "thiscall" BFResource_attempt, offset = 0x00003891)]
    fn zoo_bf_resource_attempt(this_ptr: u32, file_name: u32) -> u8 {
        if bf_resource_inner(this_ptr, file_name) {
            return 1;
        }
        unsafe { BFResource_attempt.call(this_ptr, file_name) }
    }

    //47f4
    #[hook(unsafe extern "thiscall" BFResource_prepare, offset = 0x000047f4)]
    fn zoo_bf_resource_prepare(this_ptr: u32, file_name: u32) -> u8 {
        if bf_resource_inner(this_ptr, file_name) {
            return 1;
        }

        unsafe { BFResource_prepare.call(this_ptr, file_name) }
    }

    fn bf_resource_inner(this_ptr: u32, file_name: u32) -> bool {
        let file_name_string = get_string_from_memory(file_name).to_lowercase();
        if check_file(&file_name_string)
            && let Some(ptr) = get_file_ptr(&file_name_string)
        {
            let mut bfrp = unsafe { Box::from_raw(ptr as *mut BFResourcePtr) };

            bfrp.num_refs = 100;

            let ptr = Box::into_raw(bfrp) as u32;

            save_to_memory(this_ptr, ptr);
            true
        } else {
            false
        }
    }

    #[hook(unsafe extern "thiscall" BFResourceMgr_constructor, offset = 0x0012903f)]
    fn zoo_bf_resource_mgr_constructor(this_ptr: u32) -> u32 {
        info!("BFResourceMgr::constructor({:X})", this_ptr);

        use std::time::Instant;
        let now = Instant::now();
    
        let return_value = unsafe { BFResourceMgr_constructor.call(this_ptr) };

        let elapsed = now.elapsed();
        info!("Vanilla loading took {:.2?}", elapsed);

        let ini_path = get_ini_path();
        let mut zoo_ini = Ini::new();
        zoo_ini.set_comment_symbols(&['#']);
        if let Err(e) = zoo_ini.load(ini_path) {
            info!("Failed to load zoo.ini: {}", e);
            return return_value;
        
        };
        if let Some(paths) = zoo_ini.get("resource", "path") {
            // TODO: Re-add this when more expansions can be added, expand to add subdirs of mods to ZT path variable
            // let path_vec = paths.split(';').map(|s| s.to_owned()).collect::<Vec<String>>();
            // if !path_vec.clone().into_iter().any(|s| s.trim() == "./mods") {
            //     info!("Adding mods directory to BFResourceMgr");
            //     let add_path: extern "thiscall" fn(u32, u32) -> u32 = unsafe { std::mem::transmute(0x0052870b) };
            //     if let Ok(mods_path) = CString::new("./mods") {
            //         add_path(this_ptr, mods_path.as_ptr() as u32);
            //     }
            // }
            info!("Loading resources from: {}", paths);
            load_resources(paths.split(';').map(|s| s.to_owned()).collect());
            info!("Resources loaded");
        }
        return_value
    }

    #[hook(unsafe extern "cdecl" ZTUI_general_getInfoImageName, offset = 0x000f85d2)]
    fn zoo_ui_general_get_info_image_name(param_1: u32) -> u32 {
        let return_value = unsafe { ZTUI_general_getInfoImageName.call(param_1) };
        info!("ZTUI_general_getInfoImageName({}) -> {:X} {}", param_1, return_value, get_string_from_memory(return_value));
        return_value
    }
}

#[derive(Clone)]
pub struct Handler {
    matcher_prefix: Option<String>,
    matcher_suffix: Option<String>,
    handler: HandlerFunction,
    mod_type: ModType,
}

#[derive(Clone)]
pub enum ModType {
    Legacy,
    OpenZT,
}

pub type HandlerFunction = fn(&Path, &mut ZipFile) -> ();

impl Handler {
    pub fn new(
        matcher_prefix: Option<String>,
        matcher_suffix: Option<String>,
        handler: HandlerFunction,
        mod_type: ModType,
    ) -> Self {
        Self {
            matcher_prefix,
            matcher_suffix,
            handler,
            mod_type,
        }
    }

    fn handle(&self, entry: &Path, file: &mut ZipFile) {
        let file_name = file.name();
        if let Some(prefix) = &self.matcher_prefix {
            if !file_name.starts_with(prefix) {
                return;
            }
        }
        if let Some(file_type) = &self.matcher_suffix {
            if !file_name.ends_with(file_type) {
                return;
            }
        }

        match self.mod_type {
            ModType::Legacy => {
                if file_name.ends_with(".zip") {
                    return;
                }
            }
            ModType::OpenZT => {
                if entry.to_str().unwrap_or_default().to_lowercase().ends_with(".ztd") {
                    return;
                } else {
                    info!("Loading OpenZT mod: {} file: {}", file_name, entry.display());
                }
            }
        }

        (self.handler)(entry, file);
    }
}

// Note: We are excluding ztat* files until we need to override anything inside them
fn get_ztd_resources(dir: &Path, recursive: bool) -> Vec<PathBuf> {
    let mut resources = Vec::new();
    if !dir.is_dir() {
        return resources;
    }
    let walker = WalkDir::new(dir)
        .follow_links(true)
        .max_depth(if recursive { 0 } else { 1 });
    for entry in walker {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                error!("Error walking directory: {}", e);
                continue;
            }
        };
        let Some(filename) = entry.file_name().to_str() else {
            error!("Error getting filename: {:?}", entry);
            continue;
        };
        if filename.to_lowercase().ends_with(".ztd") && !filename.starts_with("ztat") || filename.to_lowercase().ends_with(".zip") {
            resources.push(entry.path().to_path_buf());
        }
    }
    resources
}

fn load_resources(paths: Vec<String>) {
    use std::time::Instant;
    let now = Instant::now();

    paths.iter().for_each(|path| {
        let resources = get_ztd_resources(Path::new(path), false);
        resources.iter().for_each(|resource| {
            let file_name = resource.to_str().unwrap_or_default().to_lowercase();
            if file_name.ends_with(".ztd") {
                handle_ztd(resource);
            } else if file_name.ends_with(".zip") {
                handle_ztd2(resource);
            }
            // handle_ztd(resource);
        });
    });
    
    let elapsed = now.elapsed();
    info!("Loaded {} mods in: {:.2?}", get_num_resources(), elapsed);
    // list_openzt_mod_buffer();
}

// Handler V2, supporting OpenZT and legacy mods
// TODO: Benchmark reading all files initially vs reading them as needed (hypothesis: reading all at once is faster, given we likely need to read all the files eventually anyway)
fn handle_ztd2(resource: &PathBuf) {
    let file = match File::open(resource) {
        Ok(file) => file,
        Err(e) => {
            error!("Error opening file: {}", e);
            return;
        }
    };

    let mut buf_reader = BufReader::new(file);

    let mut zip = match zip::ZipArchive::new(&mut buf_reader) {
        Ok(zip) => zip,
        Err(e) => {
            error!("Error reading zip: {}", e);
            return;
        }
    };

    let mut openzt_mod = false;

    let mut file_map: HashMap<String, Box<[u8]>> = HashMap::new();
    for i in 0..zip.len() {
        let mut file = match zip.by_index(i) {
            Ok(file) => file,
            Err(e) => {
                error!("Error reading zip file: {}", e);
                continue;
            }
        };
        if file.is_dir() {
            continue;
        }
        let file_name = file.name().to_string();
        if file_name == "meta.toml" {
            openzt_mod = true;
        }

        let mut file_buffer = vec![0; file.size() as usize].into_boxed_slice();
        match file.read_exact(&mut file_buffer) {
            Ok(bytes_read) => bytes_read,
            Err(e) => {
                error!(
                    "Error reading file: {} -> {}",
                    file.name(),
                    e
                );
                continue;
            }
        };

        file_map.insert(file_name, file_buffer);
    }

    // if zipfile_map.contains_key("meta.toml") {
    if openzt_mod {
        load_open_zt_mod(file_map);
    } //TODO: Legacy mods
 
}

fn load_open_zt_mod(mut file_map: HashMap<String,Box<[u8]>>) {
    let Some(meta_file) = file_map.get_mut("meta.toml") else {
        error!("Error reading meta.toml from OpenZT mod");
        return;
    };

    let intermediate_string = String::from_utf8_lossy(&meta_file).to_string();

    let Ok(meta) = toml::from_str::<mods::Meta>(&intermediate_string) else {
        error!("Error parsing meta.toml from OpenZT mod");
        return;
    };

    let mut mod_id = meta.mod_id().to_string();

    info!("Loading OpenZT mod: {} {}", meta.name(), meta.mod_id());

    for file in file_map.keys() {
        if file.starts_with("/defs/") {
            load_defs(&mod_id, &mut file_map, &file);
        }
    }

}

static LOCATIONS_ARRAY: Lazy<Mutex<Vec<String>>> = Lazy::new(|| Mutex::new(Vec::new()));
static HABITATS_ARRAY: Lazy<Mutex<Vec<String>>> = Lazy::new(|| Mutex::new(Vec::new()));
static MOD_ID_SET: Lazy<Mutex<HashSet<String>>> = Lazy::new(|| Mutex::new(HashSet::new()));
const MIN_HABITAT_ID: u32 = 9414;
const MAX_HABITAT_ID: u32 = 9600;
const MIN_LOCATION_ID: u32 = 9634;
const MAX_LOCATION_ID: u32 = 9800;

fn add_location(location: String) {
    let Ok(mut binding) = LOCATIONS_ARRAY.lock() else {
        error!("Failed to lock locations array; returning from add_location for {}", location);
        return;
    };
    binding.push(location);
}

fn add_habitat(habitat: String) {
    let Ok(mut binding) = HABITATS_ARRAY.lock() else {
        error!("Failed to lock habitats array; returning from add_habitat for {}", habitat);
        return;
    };
    binding.push(habitat);
}

enum ResourceType {
    Location,
    Habitat
}

impl Display for ResourceType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ResourceType::Location => write!(f, "location"),
            ResourceType::Habitat => write!(f, "habitat"),
        }
    }

}

enum ZTResourceType {
    Animation,
    Ani,
    Palette
}

impl Display for ZTResourceType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ZTResourceType::Animation => write!(f, "animation"),
            ZTResourceType::Palette => write!(f, "palette"),
            ZTResourceType::Ani => write!(f, "ani"),
        }
    }
}


fn load_defs(mod_id: &String, mut file_map: &mut HashMap<String, Box<[u8]>>, def_file_name: &String) {
    info!("Loading defs from {} {}", mod_id, def_file_name);
    let Some(defs_file) = file_map.get_mut(def_file_name) else {
        error!("Error reading defs.toml from OpenZT mod");
        return;
    };

    let intermediate_string = String::from_utf8_lossy(&defs_file).to_string();
    info!("Intermediate string: {}", intermediate_string);

    let Ok(defs) = toml::from_str::<mods::ModDefinition>(&intermediate_string) else {
        error!("Error parsing defs.toml from OpenZT mod");
        return;
    };

    info!("Loading defs: {}", defs.len());

    // Habitats
    if let Some(habitats) = defs.habitats() {
        let mut habitats_binding = match HABITATS_ARRAY.lock() {
            Ok(habitats_locations) => habitats_locations,
            Err(e) => {
                error!("Error locking habitats locations array: {}", e);
                return;
            }
        };
        for (habitat_name, habitat_def) in habitats.iter() {
            let base_resource_id = openzt_base_resource_id(&mod_id, ResourceType::Habitat, habitat_name);
            // Get icon file ptr, log error and continue if not found
            let Some(icon_file) = file_map.get(habitat_def.icon_path()) else {
                error!("Error loading openzt mod {}, cannot find file {} for habitat {}", mod_id, habitat_def.icon_path(), habitat_name);
                continue;
            };
            let Some(icon_file_palette) = file_map.get(habitat_def.icon_palette_path()) else {
                error!("Error loading openzt mod {}, cannot find file {} for habitat {}", mod_id, habitat_def.icon_palette_path(), habitat_name);
                continue;
            };

            let mut animation = Animation::parse(icon_file);
            animation.set_palette_filename(habitat_def.icon_palette_path().clone());
            let (new_animation_bytes, length) = animation.write();
            let new_icon_file = new_animation_bytes.into_boxed_slice();

            let mut ani_cfg = Ini::new_cs();
            ani_cfg.set_comment_symbols(&[';', '#', ':']);
            if let Err(err) = ani_cfg.read(include_str!("../resources/include/infoimg-habitat.ani").to_string()) {
                error!("Error reading ini: {}", err);
                continue;
            };

            ani_cfg.set("Animation", "dir0", Some(openzt_full_resource_id_path(&base_resource_id, ZTResourceType::Animation)));

            let mut write_options = WriteOptions::default();
            write_options.space_around_delimiters = true;
            write_options.blank_lines_between_sections = 1;
            let new_string = ani_cfg.pretty_writes(&write_options);
            let file_size = new_string.len() as u32;
            let file_name = openzt_full_resource_id_path(&base_resource_id, ZTResourceType::Ani);

            let Ok(new_c_string) = CString::new(new_string) else {
                error!("Error loading openzt mod {} when converting .ani to CString after modifying {}", mod_id, file_name);
                continue;
            };

            let Ok(ztfile) = ZTFile::new_text(file_name.clone(), file_size, new_c_string) else {
                error!("Error loading openzt mod {} when creating ZTFile for .ani after modifying {}", mod_id, file_name);
                continue;
            };
            add_ztfile(Path::new("zip::./openzt.ztd"), file_name, ztfile)

            // add_txt_file_to_map_with_path_override(entry, file, path)
            
            // add_raw_bytes_to_map_with_path_override(entry, file, path)



        }
        // habitats_locations.extend(habitats.iter().map(|h| h.location().to_string()));
    };

    // Locations
}



fn openzt_base_resource_id(mod_id: &String, resource_type: ResourceType, resource_name: &String) -> String {
    let resource_type_name = resource_type.to_string();
    format!("{}.{}.{}", mod_id, resource_type_name, resource_name)
}

fn openzt_full_resource_id_path(base_resource_id: &String, file_type: ZTResourceType) -> String {
    format!("{}.{}", base_resource_id, file_type.to_string())
}

// fn load_habitats(mod_id: String, mut file_map: HashMap<String, Box<[u8]>>) {
//     let Some(habitat_file) = file_map.get_mut("habitats.toml") else {
//         error!("Error reading habitats.toml from OpenZT mod");
//         return;
//     };

//     let intermediate_string = String::from_utf8_lossy(&habitat_file).to_string();

//     let Ok(habitats) = toml::from_str::<mods::Habitats>(&intermediate_string) else {
//         error!("Error parsing habitats.toml from OpenZT mod");
//         return;
//     };

//     info!("Loading habitats: {}", habitats.len());
// }

fn handle_ztd(resource: &PathBuf) {
    let file = match File::open(resource) {
        Ok(file) => file,
        Err(e) => {
            error!("Error opening file: {}", e);
            return;
        }
    };

    let mut buf_reader = BufReader::new(file);

    let mut zip = match zip::ZipArchive::new(&mut buf_reader) {
        Ok(zip) => zip,
        Err(e) => {
            error!("Error reading zip: {}", e);
            return;
        }
    };
    let data_mutex = match RESOURCE_HANDLER_ARRAY.lock() {
        Ok(data_mutex) => data_mutex,
        Err(e) => {
            error!("Error locking resource handler array: {}", e);
            return;
        }
    
    };
    for handler in data_mutex.iter() {
        for i in 0..zip.len() {
            // ZipFile doesn't provide a .seek() method to set the cursor to the start of the file, so we create new ZipFile for each handler
            let mut file = match zip.by_index(i) {
                Ok(file) => file,
                Err(e) => {
                    error!("Error reading zip file: {}", e);
                    continue;
                }
            };
            if file.is_dir() {
                continue;
            }
            handler.handle(resource, &mut file);
        }
    }
}

static RESOURCE_HANDLER_ARRAY: Lazy<Mutex<Vec<Handler>>> = Lazy::new(|| Mutex::new(Vec::new()));

pub fn add_handler(handler: Handler) {
    let mut data_mutex = match RESOURCE_HANDLER_ARRAY.lock() {
        Ok(data_mutex) => data_mutex,
        Err(e) => {
            error!("Error locking resource handler array: {}", e);
            return;
        }
    };
    data_mutex.push(handler);
}

fn get_handlers() -> Vec<Handler> {
    match RESOURCE_HANDLER_ARRAY.lock() {
        Ok(binding) => binding.clone(),
        Err(e) => {
            error!("Error locking resource handler array: {}", e);
            Vec::new()
        }
    }
}
