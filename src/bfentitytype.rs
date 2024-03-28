use std::fmt::format;

use crate::{add_to_command_register, bfentitytype};
use crate::debug_dll::{get_from_memory, get_string_from_memory};
use crate::ztui::get_selected_entity_type;

// ------------ BFEntityType, Implementation, and Related Functions ------------ //

use tracing::info;

#[derive(Debug)]
#[repr(C)]
struct BFEntityType {
    pad1: [u8; 0x038], // -- padding: 56 bytes
    ncolors: u32, // 0x038
    pad2: [u8; 0x050 - 0x03C], // -- padding: 20 bytes
    icon_zoom: bool, // 0x050
    pad3: [u8; 0x054 - 0x051], // -- padding: 3 bytes
    expansion_id: bool, // 0x054
    movable: bool, // 0x055
    walkable: bool, // 0x056
    walkable_by_tall: bool, // 0x057
    pad4: [u8; 0x059 - 0x058], // -- padding: 1 byte
    rubbleable: bool, // 0x059
    pad5: [u8; 0x05B - 0x05A], // -- padding: 1 byte
    use_numbers_in_name: bool, // 0x05B
    uses_real_shadows: bool, // 0x05C
    has_shadow_images: bool, // 0x05D
    force_shadow_black: bool, // 0x05E
    pad6: [u8; 0x060 - 0x05F], // -- padding: 1 byte
    draws_late: bool, // 0x060
    pad7: [u8; 0x064 - 0x061], // -- padding: 3 bytes
    height: u32, // 0x064
    depth: u32, // 0x068
    has_underwater_section: bool, // 0x06C
    is_transient: bool, // 0x06D
    uses_placement_cube: bool, // 0x06E
    show: bool, // 0x06F
    hit_threshold: u32, // 0x070
    avoid_edges: bool, // 0x074
    pad10: [u8; 0x0B4 - 0x075], // -- padding: 47 bytes
    footprintx: i32, // 0x0B4
    footprinty: i32, // 0x0B8
    footprintz: i32, // 0x0BC
    placement_footprintx: i32, // 0x0C0
    placement_footprinty: i32, // 0x0C4
    placement_footprintz: i32, // 0x0C8
    available_at_startup: bool, // 0x0CC
    pad11: [u8; 0x100 - 0x0CD], // -- padding: 35 bytes
}

impl BFEntityType {
    // returns the instance of the BFEntityType struct
    fn new(address: u32) -> Option<&'static mut BFEntityType> {
        unsafe {
            // get the pointer to the BFEntityType instance    
            let ptr = get_from_memory::<*mut BFEntityType>(address);
    
            // is pointer not null
            if !ptr.is_null() {
                Some(&mut *ptr)
            } 
            else {
                // pointer is null
                None
            }
        }
    }

    // returns the codename of the entity type
    fn get_codename(&self) -> String {
        let obj_ptr = self as *const BFEntityType as u32;
        get_string_from_memory(get_from_memory::<u32>(obj_ptr + 0x098))
    }

    // returns the type name of the entity type
    fn get_type_name(&self) -> String {
        let obj_ptr = self as *const BFEntityType as u32;
        get_string_from_memory(get_from_memory::<u32>(obj_ptr + 0x0A4))
    }

    // allows setting the configuration of the entity type
    fn set_config(&mut self, config: &str, value: &str) -> Result<String, &'static str> {
        if config == "cIconZoom" {
            self.icon_zoom = value.parse::<bool>().unwrap();
            Ok(format!("Set cIconZoom to {}", self.icon_zoom))
        }
        else if config == "cExpansionID" {
            self.expansion_id = value.parse::<bool>().unwrap();
            Ok(format!("Set cExpansionID to {}", self.expansion_id))
        }
        else if config == "cMovable" {
            self.movable = value.parse::<bool>().unwrap();
            Ok(format!("Set cMovable to {}", self.movable))
        }
        else if config == "cWalkable" {
            self.walkable = value.parse::<bool>().unwrap();
            Ok(format!("Set cWalkable to {}", self.walkable))
        }
        else if config == "cWalkableByTall" {
            self.walkable_by_tall = value.parse::<bool>().unwrap();
            Ok(format!("Set cWalkableByTall to {}", self.walkable_by_tall))
        }
        else if config == "cRubbleable" {
            self.rubbleable = value.parse::<bool>().unwrap();
            Ok(format!("Set cRubbleable to {}", self.rubbleable))
        }
        else if config == "cUseNumbersInName" {
            self.use_numbers_in_name = value.parse::<bool>().unwrap();
            Ok(format!("Set cUseNumbersInName to {}", self.use_numbers_in_name))
        }
        else if config == "cUsesRealShadows" {
            self.uses_real_shadows = value.parse::<bool>().unwrap();
            Ok(format!("Set cUsesRealShadows to {}", self.uses_real_shadows))
        }
        else if config == "cHasShadowImages" {
            self.has_shadow_images = value.parse::<bool>().unwrap();
            Ok(format!("Set cHasShadowImages to {}", self.has_shadow_images))
        }
        else if config == "cForceShadowBlack" {
            self.force_shadow_black = value.parse::<bool>().unwrap();
            Ok(format!("Set cForceShadowBlack to {}", self.force_shadow_black))
        }
        else if config == "cDrawsLate" {
            self.draws_late = value.parse::<bool>().unwrap();
            Ok(format!("Set cDrawsLate to {}", self.draws_late))
        }
        else if config == "cHeight" {
            self.height = value.parse::<u32>().unwrap();
            Ok(format!("Set cHeight to {}", self.height))
        }
        else if config == "cDepth" {
            self.depth = value.parse::<u32>().unwrap();
            Ok(format!("Set cDepth to {}", self.depth))
        }
        else if config == "cHasUnderwaterSection" {
            self.has_underwater_section = value.parse::<bool>().unwrap();
            Ok(format!("Set cHasUnderwaterSection to {}", self.has_underwater_section))
        }
        else if config == "cIsTransient" {
            self.is_transient = value.parse::<bool>().unwrap();
            Ok(format!("Set cIsTransient to {}", self.is_transient))
        }
        else if config == "cUsesPlacementCube" {
            self.uses_placement_cube = value.parse::<bool>().unwrap();
            Ok(format!("Set cUsesPlacementCube to {}", self.uses_placement_cube))
        }
        else if config == "cShow" {
            self.show = value.parse::<bool>().unwrap();
            Ok(format!("Set cShow to {}", self.show))
        }
        else if config == "cHitThreshold" {
            self.hit_threshold = value.parse::<u32>().unwrap();
            Ok(format!("Set cHitThreshold to {}", self.hit_threshold))
        }
        else if config == "cAvoidEdges" {
            self.avoid_edges = value.parse::<bool>().unwrap();
            Ok(format!("Set cAvoidEdges to {}", self.avoid_edges))
        }
        else if config == "cFootprintX" {
            self.footprintx = value.parse::<i32>().unwrap();
            Ok(format!("Set cFootprintX to {}", self.footprintx))
        }
        else if config == "cFootprintY" {
            self.footprinty = value.parse::<i32>().unwrap();
            Ok(format!("Set cFootprintY to {}", self.footprinty))
        }
        else if config == "cFootprintZ" {
            self.footprintz = value.parse::<i32>().unwrap();
            Ok(format!("Set cFootprintZ to {}", self.footprintz))
        }
        else if config == "cPlacementFootprintX" {
            self.placement_footprintx = value.parse::<i32>().unwrap();
            Ok(format!("Set cPlacementFootprintX to {}", self.placement_footprintx))
        }
        else if config == "cPlacementFootprintY" {
            self.placement_footprinty = value.parse::<i32>().unwrap();
            Ok(format!("Set cPlacementFootprintY to {}", self.placement_footprinty))
        }
        else if config == "cPlacementFootprintZ" {
            self.placement_footprintz = value.parse::<i32>().unwrap();
            Ok(format!("Set cPlacementFootprintZ to {}", self.placement_footprintz))
        }
        else if config == "cAvailableAtStartup" {
            self.available_at_startup = value.parse::<bool>().unwrap();
            Ok(format!("Set cAvailableAtStartup to {}", self.available_at_startup))
        }
        else {
            Err("Invalid configuration option")
        }
    }

    // prints the configuration of the entity type
    fn print_config(&self) -> String {
        // NOTE: ncolors is part of a separate structure in memory withn BFEntityType, so we need to grab the pointer to it first
        // this is temporary until the struct can be fully implemented
        let entity_type_address = get_selected_entity_type(); // grab the address of the selected entity type
        let entity_type_print = get_from_memory::<u32>(entity_type_address); // convert the address to a u32 ptr for printing
        let ncolors_ptr = get_from_memory::<u32>(entity_type_print + 0x038);
        let ncolors = get_from_memory::<u32>(ncolors_ptr);

        format!("\n\n[Details]\n\nEntity Type Address: {:#x}\nType Name: {}\nCodename: {}\n\n[Configuration]\n\nncolors: {}\ncIconZoom: {}\ncExpansionID: {}\ncMovable: {}\ncWalkable: {}\ncWalkableByTall: {}\ncRubbleable: {}\ncUseNumbersInName: {}\ncUsesRealShadows: {}\ncHasShadowImages: {}\ncForceShadowBlack: {}\ncDrawsLate: {}\ncHeight: {}\ncDepth: {}\ncHasUnderwaterSection: {}\ncIsTransient: {}\ncUsesPlacementCube: {}\ncShow: {}\ncHitThreshold: {}\ncAvoidEdges: {}\ncFootprintX: {}\ncFootprintY: {}\ncFootprintZ: {}\ncPlacementFootprintX: {}\ncPlacementFootprintY: {}\ncPlacementFootprintZ: {}\ncAvailableAtStartup: {}\n",
        self as *const BFEntityType as u32,
        self.get_type_name(),
        self.get_codename(),
        ncolors,
        self.icon_zoom as u32,
        self.expansion_id as u32,
        self.movable as u32,
        self.walkable as u32,
        self.walkable_by_tall as u32,
        self.rubbleable as u32,
        self.use_numbers_in_name as u32,
        self.uses_real_shadows as u32,
        self.has_shadow_images as u32,
        self.force_shadow_black as u32,
        self.draws_late as u32,
        self.height,
        self.depth,
        self.has_underwater_section as u32,
        self.is_transient as u32,
        self.uses_placement_cube as u32,
        self.show as u32,
        self.hit_threshold,
        self.avoid_edges as u32,
        self.footprintx,
        self.footprinty,
        self.footprintz,
        self.placement_footprintx,
        self.placement_footprinty,
        self.placement_footprintz,
        self.available_at_startup as u32,
        )
    }
}

// returns the selected entity type
// usage: selected_type [-v] returns printed configuration of the selected entity type and other details
// usage: selected_type [-<config> <value>] sets the configuration of the selected entity type
pub fn command_selected_type(_args: Vec<&str>) -> Result<String, &'static str> {
    
    let entity_type_address = get_selected_entity_type(); // grab the address of the selected entity type
    let entity_type_print = get_from_memory::<u32>(entity_type_address); // convert the address to a u32 ptr for printing
    if entity_type_address == 0 {
        return Err("No entity selected");
    }
    let entity_type = BFEntityType::new(entity_type_address).unwrap(); // create a copied instance of the entity type

    // if no selected entity type, return error
    if _args.len() == 0 {
        return Ok(format!("\n[Details]\n\nEntity Type Address: {:#x}\nType Name: {}\nCodename: {}\n\n", entity_type_print, entity_type.get_type_name(), entity_type.get_codename()));
    }

    // if -v flag is used, print the entity type configuration and other details
    if _args[0] == "-v" {

        info!("Printing configuration for entity type at address {:#x}", entity_type_print);
        
        // print the entity type configuration
        Ok(entity_type.print_config())
    }
    else if _args.len() == 2 {
        entity_type.set_config(_args[0], _args[1]);
        Ok("\n[/End of configuration]".to_string())
    }
    else {
        Ok("Invalid argument".to_string())
    }
}

pub fn init() {
    add_to_command_register("selected_type".to_string(), command_selected_type);
    add_to_command_register("selected_scenery".to_string(), command_selected_scenery);
}


// ------------ ZTSceneryType, Implementation, and Related Functions ------------ //

#[derive(Debug)]
#[repr(C)]
struct ZTSceneryType {
    bfentitytype: BFEntityType, // 0x000
    purchase_cost: f32, // 0x100
    name_id: u32, // 0x104
    help_id: u32, // 0x108
    habitat: u32, // 0x10C
    location: u32, // 0x110
    era: u32, // 0x114
    max_food_units: u32, // 0x118
    stink: bool, // 0x11C
    pad3: [u8; 0x120 - 0x11D], // -- padding: 3 bytes
    esthetic_weight: u32, // 0x120
    pad4: [u8; 0x128 - 0x124], // -- padding: 4 bytes
    selectable: bool, // 0x128
    deletable: bool, // 0x129
    foliage: bool, // 0x12A
    pad6: [u8; 0x12D - 0x12B], // -- padding: 2 bytes
    auto_rotate: bool, // 0x12D
    land: bool, // 0x12E
    swims: bool, // 0x12F
    underwater: bool, // 0x130
    surface: bool, // 0x131
    submerge: bool, // 0x132
    only_swims: bool, // 0x133
    needs_confirm: bool, // 0x134
    gawk_only_from_front: bool, // 0x135
    dead_on_land: bool, // 0x136
    dead_on_flat_water: bool, // 0x137
    dead_underwater: bool, // 0x138
    uses_tree_rubble: bool, // 0x139
    forces_scenery_rubble: bool, // 0x13A
    blocks_los: bool, // 0x13B
}

impl ZTSceneryType {
    fn new (address: u32) -> Option<&'static mut ZTSceneryType> {
        unsafe {
            let ptr = get_from_memory::<*mut ZTSceneryType>(address);
            if !ptr.is_null() {
                Some(&mut *ptr)
            }
            else {
                None
            }
        }
    }

    fn get_info_image_name(&self) -> String {
        let obj_ptr = self as *const ZTSceneryType as u32;
        get_string_from_memory(get_from_memory::<u32>(obj_ptr + 0x14C))
    }

    fn set_config(&mut self, config: &str, value: &str) -> Result<String, &'static str> {

        if config == "-cPurchaseCost" {
            self.purchase_cost = value.parse::<f32>().unwrap();
            Ok(format!("Set Purchase Cost to {}", self.purchase_cost))
        }
        else if config == "-cNameID" {
            self.name_id = value.parse::<u32>().unwrap();
            Ok(format!("Set Name ID to {}", self.name_id))
        }
        else if config == "-cHelpID" {
            self.help_id = value.parse::<u32>().unwrap();
            Ok(format!("Set Help ID to {}", self.help_id))
        }
        else if config == "-cHabitat" {
            self.habitat = value.parse::<u32>().unwrap();
            Ok(format!("Set Habitat to {}", self.habitat))
        }
        else if config == "-cLocation" {
            self.location = value.parse::<u32>().unwrap();
            Ok(format!("Set Location to {}", self.location))
        }
        else if config == "-cEra" {
            self.era = value.parse::<u32>().unwrap();
            Ok(format!("Set Era to {}", self.era))
        }
        else if config == "-cMaxFoodUnits" {
            self.max_food_units = value.parse::<u32>().unwrap();
            Ok(format!("Set Max Food Units to {}", self.max_food_units))
        }
        else if config == "-cStink" {
            self.stink = value.parse::<bool>().unwrap();
            Ok(format!("Set Stink to {}", self.stink))
        }
        else if config == "-cEstheticWeight" {
            self.esthetic_weight = value.parse::<u32>().unwrap();
            Ok(format!("Set Esthetic Weight to {}", self.esthetic_weight))
        }
        else if config == "-cSelectable" {
            self.selectable = value.parse::<bool>().unwrap();
            Ok(format!("Set Selectable to {}", self.selectable))
        }
        else if config == "-cDeletable" {
            self.deletable = value.parse::<bool>().unwrap();
            Ok(format!("Set Deletable to {}", self.deletable))
        }
        else if config == "-cFoliage" {
            self.foliage = value.parse::<bool>().unwrap();
            Ok(format!("Set Foliage to {}", self.foliage))
        }
        else if config == "-cAutoRotate" {
            self.auto_rotate = value.parse::<bool>().unwrap();
            Ok(format!("Set Auto Rotate to {}", self.auto_rotate))
        }
        else if config == "-cLand" {
            self.land = value.parse::<bool>().unwrap();
            Ok(format!("Set Land to {}", self.land))
        }
        else if config == "-cSwims" {
            self.swims = value.parse::<bool>().unwrap();
            Ok(format!("Set Swims to {}", self.swims))
        }
        else if config == "-cUnderwater" {
            self.underwater = value.parse::<bool>().unwrap();
            Ok(format!("Set Underwater to {}", self.underwater))
        }
        else if config == "-cSurface" {
            self.surface = value.parse::<bool>().unwrap();
            Ok(format!("Set Surface to {}", self.surface))
        }
        else if config == "-cSubmerge" {
            self.submerge = value.parse::<bool>().unwrap();
            Ok(format!("Set Submerge to {}", self.submerge))
        }
        else if config == "-cOnlySwims" {
            self.only_swims = value.parse::<bool>().unwrap();
            Ok(format!("Set Only Swims to {}", self.only_swims))
        }
        else if config == "-cNeedsConfirm" {
            self.needs_confirm = value.parse::<bool>().unwrap();
            Ok(format!("Set Needs Confirm to {}", self.needs_confirm))
        }
        else if config == "-cGawkOnlyFromFront" {
            self.gawk_only_from_front = value.parse::<bool>().unwrap();
            Ok(format!("Set Gawk Only From Front to {}", self.gawk_only_from_front))
        }
        else if config == "-cDeadOnLand" {
            self.dead_on_land = value.parse::<bool>().unwrap();
            Ok(format!("Set Dead On Land to {}", self.dead_on_land))
        }
        else if config == "-cDeadOnFlatWater" {
            self.dead_on_flat_water = value.parse::<bool>().unwrap();
            Ok(format!("Set Dead On Flat Water to {}", self.dead_on_flat_water))
        }
        else if config == "-cDeadUnderwater" {
            self.dead_underwater = value.parse::<bool>().unwrap();
            Ok(format!("Set Dead Underwater to {}", self.dead_underwater))
        }
        else if config == "-cUsesTreeRubble" {
            self.uses_tree_rubble = value.parse::<bool>().unwrap();
            Ok(format!("Set Uses Tree Rubble to {}", self.uses_tree_rubble))
        }
        else if config == "-cForcesSceneryRubble" {
            self.forces_scenery_rubble = value.parse::<bool>().unwrap();
            Ok(format!("Set Forces Scenery Rubble to {}", self.forces_scenery_rubble))
        }
        else if config == "-cBlocksLOS" {
            self.blocks_los = value.parse::<bool>().unwrap();
            Ok(format!("Set Blocks LOS to {}", self.blocks_los))
        }
        else {
            Err("Invalid configuration option")
        }
    }

    fn print_config(&self) -> String {
        format!("cPurchaseCost: {}\ncNameID: {}\ncHelpID: {}\ncHabitat: {}\ncLocation: {}\ncEra: {}\ncMaxFoodUnits: {}\ncStink: {}\ncEstheticWeight: {}\ncSelectable: {}\ncDeletable: {}\ncFoliage: {}\ncAutoRotate: {}\ncLand: {}\ncSwims: {}\ncUnderwater: {}\ncSurface: {}\ncSubmerge: {}\ncOnlySwims: {}\ncNeedsConfirm: {}\ncGawkOnlyFromFront: {}\ncDeadOnLand: {}\ncDeadOnFlatWater: {}\ncDeadUnderwater: {}\ncUsesTreeRubble: {}\ncForcesSceneryRubble: {}\ncBlocksLOS: {}\n",
        self.purchase_cost,
        self.name_id,
        self.help_id,
        self.habitat,
        self.location,
        self.era,
        self.max_food_units,
        self.stink as u32,
        self.esthetic_weight,
        self.selectable as u32,
        self.deletable as u32,
        self.foliage as u32,
        self.auto_rotate as u32,
        self.land as u32,
        self.swims as u32,
        self.underwater as u32,
        self.surface as u32,
        self.submerge as u32,
        self.only_swims as u32,
        self.needs_confirm as u32,
        self.gawk_only_from_front as u32,
        self.dead_on_land as u32,
        self.dead_on_flat_water as u32,
        self.dead_underwater as u32,
        self.uses_tree_rubble as u32,
        self.forces_scenery_rubble as u32,
        self.blocks_los as u32,
        )
    }
} 

fn command_selected_scenery(_args: Vec<&str>) -> Result<String, &'static str> {
    let entity_type_address = get_selected_entity_type(); // grab the address of the selected entity type
    let entity_type_print = get_from_memory::<u32>(entity_type_address); // convert the address to a u32 ptr for printing
    if entity_type_address == 0 {
        return Err("No entity selected");
    }
    let scenery_type = ZTSceneryType::new(entity_type_address).unwrap(); // create a copied instance of the entity type

    // if no selected entity type, return error
    if _args.len() == 0 {
        return Ok(format!("\n[Details]\n\nEntity Type Address: {:#x}\nType Name: {}\nCodename: {}\n\n", entity_type_print, scenery_type.bfentitytype.get_type_name(), scenery_type.bfentitytype.get_codename()));
    }

    // if -v flag is used, print the entity type configuration and other details
    if _args[0] == "-v" {

        info!("Printing configuration for entity type at address {:#x}", entity_type_print);

        Ok(scenery_type.bfentitytype.print_config() + &scenery_type.print_config())
    }
    else if _args.len() == 2 {
        let result_entity_type = scenery_type.bfentitytype.set_config(_args[0], _args[1]);
        let result_scenery_type = scenery_type.set_config(_args[0], _args[1]);
        if result_entity_type.is_ok() {
            return result_entity_type;
        }
        else if result_scenery_type.is_ok() {
            return result_scenery_type;
        }
        else {
            return Err("Invalid configuration option");
        }
    }
    else {
        Ok("Invalid argument".to_string())
    }
}