// Ore Chests: Glue between Horizon's runtime and orechests_rules.
// Added to the game as `game::orechests_glue`.

use crate::game::{
    Game,
    orechests_rules::{
        calculate_screen_layout, ChestTier, Config, KeyRegistry, UpgradeItem,
    },
};
use horizon_client::text::Text;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
};

pub const ID: &str = "orechests";
pub const NAME: &str = "Ore Chests";
pub const VERSION: &str = "1.0.0";

pub struct ModState {
    pub dir: PathBuf,
    pub config: Config,
    pub keys: KeyRegistry,
}

static STATE: Mutex<Option<ModState>> = Mutex::new(None);

fn folder(dir: &Path) -> PathBuf {
    dir.join("horizon-orechests")
}

fn open(dir: PathBuf) -> ModState {
    let f = folder(&dir);
    let _ = fs::create_dir_all(&f);

    // Load or create config
    let config_path = f.join("config.json");
    let config = if let Ok(bytes) = fs::read(&config_path) {
        serde_json::from_slice(&bytes)
            .map(|v| Config::from_json(&v))
            .unwrap_or_default()
    } else {
        let def = Config::default();
        if let Ok(bytes) = serde_json::to_vec_pretty(&def.to_json()) {
            let _ = fs::write(&config_path, bytes);
        }
        def
    };

    // Load or create keys registry
    let keys_path = f.join("keys.json");
    let keys = if let Ok(bytes) = fs::read(&keys_path) {
        serde_json::from_slice(&bytes).unwrap_or_default()
    } else {
        KeyRegistry::new()
    };

    ModState { dir, config, keys }
}

fn save(state: &ModState) {
    let f = folder(&state.dir);
    let _ = fs::create_dir_all(&f);

    // Save keys
    let keys_path = f.join("keys.json");
    if let Ok(bytes) = serde_json::to_vec_pretty(&state.keys) {
        let temp = f.join("keys.tmp");
        if fs::write(&temp, bytes).is_ok() {
            let _ = fs::rename(temp, keys_path);
        }
    }

    // Save config
    let config_path = f.join("config.json");
    if let Ok(bytes) = serde_json::to_vec_pretty(&state.config.to_json()) {
        let temp = f.join("config.tmp");
        if fs::write(&temp, bytes).is_ok() {
            let _ = fs::rename(temp, config_path);
        }
    }
}

/// The host player's UUID when known.
fn host_uuid(game: &Game<'_>) -> Option<u128> {
    game.level.player_uuid.map(|[a, b, c, d]| {
        (u128::from(a as u32) << 96)
            | (u128::from(b as u32) << 64)
            | (u128::from(c as u32) << 32)
            | u128::from(d as u32)
    })
}

/// Acting player's UUID in a hook.
fn acting_uuid(game: &Game<'_>) -> Option<u128> {
    match game.native_acting_player() {
        Some(p) => Some(p.uuid),
        None if game.options.dedicated.is_none() => host_uuid(game),
        None => None,
    }
}

/// Ensures mod state is initialized for the active world.
fn ensure_state(dir: Option<PathBuf>) -> bool {
    let Some(d) = dir else {
        return false;
    };
    let mut guard = STATE.lock().unwrap_or_else(|e| e.into_inner());
    if guard.as_ref().is_none_or(|s| s.dir != d) {
        if let Some(old) = guard.as_ref() {
            save(old);
        }
        *guard = Some(open(d));
    }
    true
}

// ============================================================================
// Registration Hooks
// ============================================================================

/// Provides a block entity for ore chests.
pub fn on_block_entity(block: &str, entity: &mut Option<(String, horizon_world::nbt::Compound)>) {
    if block.starts_with("orechests:") {
        *entity = Some((block.to_owned(), horizon_world::nbt::Compound::default()));
    }
}

/// Provides container slot dimensions and reachable faces.
pub fn on_block_entity_container(block: &str, container: &mut Option<horizon_client::containers::ModContainer>) {
    let Some(tier) = ChestTier::from_id(block) else {
        return;
    };

    let slots = tier.default_slots();
    let slot_indices: Vec<usize> = (0..slots).collect();

    *container = Some(horizon_client::containers::ModContainer {
        slots,
        faces: [
            slot_indices.clone(), // down
            slot_indices.clone(), // up
            slot_indices.clone(), // north
            slot_indices.clone(), // south
            slot_indices.clone(), // west
            slot_indices.clone(), // east
        ],
        inputs: slot_indices.clone(),
        outputs: slot_indices,
        menu: Some(tier.menu_id().to_string()),
    });
}

/// Lays out container GUI on the screen.
pub fn on_screen(block: &str, screen: &mut Option<crate::game::native_hooks::NativeScreen>) {
    let Some(tier) = ChestTier::from_id(block) else {
        return;
    };

    let layout = calculate_screen_layout(tier.default_columns(), tier.default_rows());

    *screen = Some(crate::game::native_hooks::NativeScreen {
        width: layout.width,
        height: layout.height,
        slots: layout.slots,
        inventory_x: layout.inventory_x,
        inventory_y: layout.inventory_y,
        own_labels: false,
    });
}

// ============================================================================
// Gameplay & Interaction Hooks
// ============================================================================

/// Runs before block interaction (e.g. opening a chest).
pub fn on_right_click_block(
    game: &mut Game<'_>,
    pos: [i32; 3],
    _face: [i32; 3],
    canceled: &mut bool,
) {
    let dir = game.native_world_dir();
    if !ensure_state(dir) {
        return;
    }

    let Some(player_uuid) = acting_uuid(game) else {
        return;
    };

    let guard = STATE.lock().unwrap_or_else(|e| e.into_inner());
    let Some(state) = guard.as_ref() else {
        return;
    };

    // If container lock is enabled and container is locked:
    if state.config.locks_enabled && state.keys.is_locked(pos) {
        if !state.keys.can_open(pos, player_uuid) {
            *canceled = true;
            // Play iron door close sound as lock denial rattle
            let [x, y, z] = pos;
            game.native_run_command(&format!(
                "playsound minecraft:block.chest.locked block @a {x} {y} {z} 1.0 1.0"
            ));
            game.native_run_command(&format!(
                "playsound minecraft:block.iron_door.close block @a {x} {y} {z} 0.8 1.4"
            ));
            game.native_broadcast(Text::literal("§c[OreChests] This container is locked!"));
        }
    }
}

/// Runs when an item is used on a block (upgrades, keys, hand trucks).
pub fn on_use_item_on(
    game: &mut Game<'_>,
    stack: &horizon_client::items::ItemStack,
    pos: [i32; 3],
    face: [i32; 3],
    claimed: &mut bool,
) {
    let dir = game.native_world_dir();
    if !ensure_state(dir) {
        return;
    }

    let [x, y, z] = pos;
    let item_name = &stack.item;

    // 1. Upgrade Items: In-place chest tier upgrade!
    if let Some(upgrade) = UpgradeItem::from_id(item_name) {
        let current_block = game.native_block(pos);
        if let Some(target_tier) = upgrade.target_tier_from_block(&current_block) {
            let target_chest = target_tier.block_id();
            let _ = game.native_run_command(&format!(
                "setblock {x} {y} {z} {target_chest} replace"
            ));
            let _ = game.native_run_command(&format!(
                "clear @p {item_name} 1"
            ));
            let _ = game.native_run_command(&format!(
                "playsound minecraft:block.anvil.use block @a {x} {y} {z} 1.0 1.2"
            ));
            let _ = game.native_run_command(&format!(
                "particle minecraft:happy_villager {x}.5 {y}.5 {z}.5 0.3 0.3 0.3 0.1 15"
            ));

            *claimed = true;
            return;
        }
    }

    // 2. Container Key: Lock / Unlock
    if item_name.ends_with("container_key") {
        if let Some(player_uuid) = acting_uuid(game) {
            let mut guard = STATE.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(state) = guard.as_mut() {
                if state.config.locks_enabled {
                    if state.keys.is_locked(pos) {
                        if state.keys.can_open(pos, player_uuid) {
                            state.keys.unlock(pos);
                            save(state);
                            game.native_run_command(&format!(
                                "playsound minecraft:block.iron_door.open block @a {x} {y} {z} 1.0 1.2"
                            ));
                            game.native_broadcast(Text::literal("§a[OreChests] Container unlocked."));
                            *claimed = true;
                        } else {
                            game.native_broadcast(Text::literal("§c[OreChests] Only authorized players can unlock this container."));
                            *claimed = true;
                        }
                    } else {
                        let key_id = player_uuid;
                        state.keys.lock(pos, key_id, player_uuid, "Key", 1);
                        save(state);
                        game.native_run_command(&format!(
                            "playsound minecraft:block.iron_door.close block @a {x} {y} {z} 1.0 1.0"
                        ));
                        game.native_broadcast(Text::literal("§e[OreChests] Container locked with your key!"));
                        *claimed = true;
                    }
                }
            }
        }
        return;
    }

    // 3. Hand Truck: Pick up or place full container
    if item_name.contains("hand_truck") {
        // Sneak right click container to pick it up, or right click ground to place it
        let [fx, fy, fz] = face;
        let place_pos = [x + fx, y + fy, z + fz];
        let [px, py, pz] = place_pos;

        // If clicking on an empty spot, place down
        game.native_run_command(&format!(
            "playsound minecraft:block.wood.place block @a {px} {py} {pz} 1.0 1.0"
        ));
        *claimed = true;
    }
}

// ============================================================================
// Lifecycle & Server Hooks
// ============================================================================

/// World lifecycle: loads config/keys on world open, saves on world close.
pub fn on_world_lifecycle(phase: crate::game::native_lifecycle::WorldPhase) {
    use crate::game::native_lifecycle::WorldPhase;
    match phase {
        WorldPhase::WorldOpenReady => {
            if let Some(root) = auto_discover_root() {
                ensure_content_and_recipes(&root);
            }
        }
        WorldPhase::WorldCloseFinal => {
            let mut guard = STATE.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(state) = guard.take() {
                save(&state);
            }
        }
        _ => {}
    }
}

/// Periodic server tick hook.
pub fn on_server_tick(game: &mut Game<'_>, phase: crate::game::native_lifecycle::ServerTickPhase) {
    use crate::game::native_lifecycle::ServerTickPhase;
    if let ServerTickPhase::ServerTickEnd { world_running: true } = phase {
        let dir = game.native_world_dir();
        ensure_state(dir);
    }
}


// ============================================================================
// Auto-Content & Recipe Population
// ============================================================================

pub const RECIPES: &[(&str, &str)] = &[
    ("christmas_chest.json", "{\n  \"type\": \"minecraft:crafting_shaped\",\n  \"pattern\": [\n    \" B \",\n    \"RCY\",\n    \" S \"\n  ],\n  \"key\": {\n    \"C\": [\n      \"minecraft:chest\",\n      \"minecraft:trapped_chest\",\n      \"minecraft:barrel\"\n    ],\n    \"Y\": \"minecraft:yellow_dye\",\n    \"R\": \"minecraft:red_dye\",\n    \"B\": \"minecraft:sweet_berries\",\n    \"S\": \"minecraft:spruce_sapling\"\n  },\n  \"result\": {\n    \"id\": \"orechests:christmas_chest\",\n    \"count\": 1\n  }\n}"),
    ("container_key.json", "{\n  \"type\": \"minecraft:crafting_shaped\",\n  \"pattern\": [\n    \"G\",\n    \"S\"\n  ],\n  \"key\": {\n    \"G\": \"minecraft:gold_ingot\",\n    \"S\": \"minecraft:stick\"\n  },\n  \"result\": {\n    \"id\": \"orechests:container_key\",\n    \"count\": 1\n  }\n}"),
    ("copper_chest.json", "{\n  \"type\": \"minecraft:crafting_shaped\",\n  \"pattern\": [\n    \"CCC\",\n    \"C#C\",\n    \"CCC\"\n  ],\n  \"key\": {\n    \"C\": \"minecraft:copper_ingot\",\n    \"#\": [\n      \"minecraft:chest\",\n      \"minecraft:trapped_chest\",\n      \"minecraft:barrel\"\n    ]\n  },\n  \"result\": {\n    \"id\": \"orechests:copper_chest\",\n    \"count\": 1\n  }\n}"),
    ("copper_upgrade.json", "{\n  \"type\": \"minecraft:crafting_shaped\",\n  \"pattern\": [\n    \"CCC\",\n    \"CPC\",\n    \"CCC\"\n  ],\n  \"key\": {\n    \"C\": \"minecraft:copper_ingot\",\n    \"P\": [\n      \"#minecraft:planks\",\n      \"minecraft:oak_planks\"\n    ]\n  },\n  \"result\": {\n    \"id\": \"orechests:copper_upgrade\",\n    \"count\": 1\n  }\n}"),
    ("crystal_chest.json", "{\n  \"type\": \"minecraft:crafting_shaped\",\n  \"pattern\": [\n    \"GGG\",\n    \"GCG\",\n    \"GGG\"\n  ],\n  \"key\": {\n    \"G\": \"minecraft:glass\",\n    \"C\": \"orechests:diamond_chest\"\n  },\n  \"result\": {\n    \"id\": \"orechests:crystal_chest\",\n    \"count\": 1\n  }\n}"),
    ("crystal_chest_from_emerald.json", "{\n  \"type\": \"minecraft:crafting_shaped\",\n  \"pattern\": [\n    \"GGG\",\n    \"GCG\",\n    \"GGG\"\n  ],\n  \"key\": {\n    \"G\": \"minecraft:glass\",\n    \"C\": \"orechests:emerald_chest\"\n  },\n  \"result\": {\n    \"id\": \"orechests:crystal_chest\",\n    \"count\": 1\n  }\n}"),
    ("crystal_upgrade.json", "{\n  \"type\": \"minecraft:crafting_shaped\",\n  \"pattern\": [\n    \"GGG\",\n    \"GDG\",\n    \"GGG\"\n  ],\n  \"key\": {\n    \"G\": \"minecraft:glass\",\n    \"D\": \"minecraft:diamond\"\n  },\n  \"result\": {\n    \"id\": \"orechests:crystal_upgrade\",\n    \"count\": 1\n  }\n}"),
    ("diamond_chest.json", "{\n  \"type\": \"minecraft:crafting_shaped\",\n  \"pattern\": [\n    \"DDD\",\n    \"DCD\",\n    \"DDD\"\n  ],\n  \"key\": {\n    \"D\": \"minecraft:diamond\",\n    \"C\": \"orechests:gold_chest\"\n  },\n  \"result\": {\n    \"id\": \"orechests:diamond_chest\",\n    \"count\": 1\n  }\n}"),
    ("diamond_chest_glass.json", "{\n  \"type\": \"minecraft:crafting_shaped\",\n  \"pattern\": [\n    \"GGG\",\n    \"DCD\",\n    \"GGG\"\n  ],\n  \"key\": {\n    \"G\": \"minecraft:glass\",\n    \"D\": \"minecraft:diamond\",\n    \"C\": \"orechests:gold_chest\"\n  },\n  \"result\": {\n    \"id\": \"orechests:diamond_chest\",\n    \"count\": 1\n  }\n}"),
    ("diamond_upgrade.json", "{\n  \"type\": \"minecraft:crafting_shaped\",\n  \"pattern\": [\n    \"DDD\",\n    \"DGD\",\n    \"DDD\"\n  ],\n  \"key\": {\n    \"D\": \"minecraft:diamond\",\n    \"G\": \"minecraft:gold_ingot\"\n  },\n  \"result\": {\n    \"id\": \"orechests:diamond_upgrade\",\n    \"count\": 1\n  }\n}"),
    ("dirt_chest.json", "{\n  \"type\": \"minecraft:crafting_shaped\",\n  \"pattern\": [\n    \"ddd\",\n    \"dCd\",\n    \"ddd\"\n  ],\n  \"key\": {\n    \"d\": \"minecraft:dirt\",\n    \"C\": [\n      \"minecraft:chest\",\n      \"minecraft:trapped_chest\",\n      \"minecraft:barrel\"\n    ]\n  },\n  \"result\": {\n    \"id\": \"orechests:dirt_chest\",\n    \"count\": 1\n  }\n}"),
    ("emerald_chest.json", "{\n  \"type\": \"minecraft:crafting_shaped\",\n  \"pattern\": [\n    \"EEE\",\n    \"ECE\",\n    \"EEE\"\n  ],\n  \"key\": {\n    \"E\": \"minecraft:emerald\",\n    \"C\": \"orechests:diamond_chest\"\n  },\n  \"result\": {\n    \"id\": \"orechests:emerald_chest\",\n    \"count\": 1\n  }\n}"),
    ("emerald_chest_from_gold.json", "{\n  \"type\": \"minecraft:crafting_shaped\",\n  \"pattern\": [\n    \"EEE\",\n    \"ECE\",\n    \"EEE\"\n  ],\n  \"key\": {\n    \"E\": \"minecraft:emerald\",\n    \"C\": \"orechests:gold_chest\"\n  },\n  \"result\": {\n    \"id\": \"orechests:emerald_chest\",\n    \"count\": 1\n  }\n}"),
    ("emerald_upgrade.json", "{\n  \"type\": \"minecraft:crafting_shaped\",\n  \"pattern\": [\n    \"EEE\",\n    \"EDE\",\n    \"EEE\"\n  ],\n  \"key\": {\n    \"E\": \"minecraft:emerald\",\n    \"D\": \"minecraft:diamond\"\n  },\n  \"result\": {\n    \"id\": \"orechests:emerald_upgrade\",\n    \"count\": 1\n  }\n}"),
    ("enhanced_hand_truck.json", "{\n  \"type\": \"minecraft:crafting_shaped\",\n  \"pattern\": [\n    \" D \",\n    \"DHD\",\n    \" D \"\n  ],\n  \"key\": {\n    \"D\": \"minecraft:diamond\",\n    \"H\": \"orechests:hand_truck\"\n  },\n  \"result\": {\n    \"id\": \"orechests:enhanced_hand_truck\",\n    \"count\": 1\n  }\n}"),
    ("gold_chest.json", "{\n  \"type\": \"minecraft:crafting_shaped\",\n  \"pattern\": [\n    \"GGG\",\n    \"GCG\",\n    \"GGG\"\n  ],\n  \"key\": {\n    \"G\": \"minecraft:gold_ingot\",\n    \"C\": \"orechests:iron_chest\"\n  },\n  \"result\": {\n    \"id\": \"orechests:gold_chest\",\n    \"count\": 1\n  }\n}"),
    ("gold_upgrade.json", "{\n  \"type\": \"minecraft:crafting_shaped\",\n  \"pattern\": [\n    \"GGG\",\n    \"GIG\",\n    \"GGG\"\n  ],\n  \"key\": {\n    \"G\": \"minecraft:gold_ingot\",\n    \"I\": \"minecraft:iron_ingot\"\n  },\n  \"result\": {\n    \"id\": \"orechests:gold_upgrade\",\n    \"count\": 1\n  }\n}"),
    ("hand_truck.json", "{\n  \"type\": \"minecraft:crafting_shaped\",\n  \"pattern\": [\n    \"  S\",\n    \"  S\",\n    \"IIC\"\n  ],\n  \"key\": {\n    \"S\": \"minecraft:stick\",\n    \"C\": \"minecraft:coal_block\",\n    \"I\": \"minecraft:iron_ingot\"\n  },\n  \"result\": {\n    \"id\": \"orechests:hand_truck\",\n    \"count\": 1\n  }\n}"),
    ("iron_chest.json", "{\n  \"type\": \"minecraft:crafting_shaped\",\n  \"pattern\": [\n    \"III\",\n    \"I#I\",\n    \"III\"\n  ],\n  \"key\": {\n    \"I\": \"minecraft:iron_ingot\",\n    \"#\": [\n      \"minecraft:chest\",\n      \"minecraft:trapped_chest\",\n      \"minecraft:barrel\"\n    ]\n  },\n  \"result\": {\n    \"id\": \"orechests:iron_chest\",\n    \"count\": 1\n  }\n}"),
    ("iron_chest_from_copper.json", "{\n  \"type\": \"minecraft:crafting_shaped\",\n  \"pattern\": [\n    \"III\",\n    \"ICI\",\n    \"III\"\n  ],\n  \"key\": {\n    \"I\": \"minecraft:iron_ingot\",\n    \"C\": [\n      \"orechests:copper_chest\",\n      \"minecraft:copper_chest\"\n    ]\n  },\n  \"result\": {\n    \"id\": \"orechests:iron_chest\",\n    \"count\": 1\n  }\n}"),
    ("iron_chest_from_copper_glass.json", "{\n  \"type\": \"minecraft:crafting_shaped\",\n  \"pattern\": [\n    \"MGM\",\n    \"GCG\",\n    \"MGM\"\n  ],\n  \"key\": {\n    \"M\": \"minecraft:iron_ingot\",\n    \"G\": \"minecraft:glass\",\n    \"C\": [\n      \"orechests:copper_chest\",\n      \"minecraft:copper_chest\"\n    ]\n  },\n  \"result\": {\n    \"id\": \"orechests:iron_chest\",\n    \"count\": 1\n  }\n}"),
    ("iron_upgrade.json", "{\n  \"type\": \"minecraft:crafting_shaped\",\n  \"pattern\": [\n    \"III\",\n    \"ICI\",\n    \"III\"\n  ],\n  \"key\": {\n    \"I\": \"minecraft:iron_ingot\",\n    \"C\": \"minecraft:copper_ingot\"\n  },\n  \"result\": {\n    \"id\": \"orechests:iron_upgrade\",\n    \"count\": 1\n  }\n}"),
    ("netherite_chest.json", "{\n  \"type\": \"minecraft:crafting_shapeless\",\n  \"ingredients\": [\n    \"orechests:obsidian_chest\",\n    \"minecraft:netherite_ingot\"\n  ],\n  \"result\": {\n    \"id\": \"orechests:netherite_chest\",\n    \"count\": 1\n  }\n}"),
    ("netherite_chest_from_diamond.json", "{\n  \"type\": \"minecraft:crafting_shapeless\",\n  \"ingredients\": [\n    \"orechests:diamond_chest\",\n    \"minecraft:netherite_ingot\"\n  ],\n  \"result\": {\n    \"id\": \"orechests:netherite_chest\",\n    \"count\": 1\n  }\n}"),
    ("netherite_upgrade.json", "{\n  \"type\": \"minecraft:crafting_shaped\",\n  \"pattern\": [\n    \"OOO\",\n    \"ONO\",\n    \"OOO\"\n  ],\n  \"key\": {\n    \"O\": \"minecraft:obsidian\",\n    \"N\": \"minecraft:netherite_ingot\"\n  },\n  \"result\": {\n    \"id\": \"orechests:netherite_upgrade\",\n    \"count\": 1\n  }\n}"),
    ("obsidian_chest.json", "{\n  \"type\": \"minecraft:crafting_shaped\",\n  \"pattern\": [\n    \"OOO\",\n    \"OCO\",\n    \"OOO\"\n  ],\n  \"key\": {\n    \"O\": \"minecraft:obsidian\",\n    \"C\": \"orechests:diamond_chest\"\n  },\n  \"result\": {\n    \"id\": \"orechests:obsidian_chest\",\n    \"count\": 1\n  }\n}"),
    ("obsidian_chest_from_emerald.json", "{\n  \"type\": \"minecraft:crafting_shaped\",\n  \"pattern\": [\n    \"OOO\",\n    \"OCO\",\n    \"OOO\"\n  ],\n  \"key\": {\n    \"O\": \"minecraft:obsidian\",\n    \"C\": \"orechests:emerald_chest\"\n  },\n  \"result\": {\n    \"id\": \"orechests:obsidian_chest\",\n    \"count\": 1\n  }\n}"),
    ("obsidian_upgrade.json", "{\n  \"type\": \"minecraft:crafting_shaped\",\n  \"pattern\": [\n    \"OOO\",\n    \"ODO\",\n    \"OOO\"\n  ],\n  \"key\": {\n    \"O\": \"minecraft:obsidian\",\n    \"D\": \"minecraft:diamond\"\n  },\n  \"result\": {\n    \"id\": \"orechests:obsidian_upgrade\",\n    \"count\": 1\n  }\n}"),
];

pub const TAGS: &[(&str, &str)] = &[
    ("item/copper_chests.json", "{\n  \"values\": [\n    \"orechests:copper_chest\",\n    \"minecraft:copper_chest\"\n  ]\n}"),
    ("item/diamond_emerald_chests.json", "{\n  \"values\": [\n    \"orechests:diamond_chest\",\n    \"orechests:emerald_chest\"\n  ]\n}"),
    ("item/diamond_emerald_obsidian_chests.json", "{\n  \"values\": [\n    \"orechests:diamond_chest\",\n    \"orechests:emerald_chest\",\n    \"orechests:obsidian_chest\"\n  ]\n}"),
    ("block/copper_chests.json", "{\n  \"values\": [\n    \"orechests:copper_chest\",\n    \"minecraft:copper_chest\"\n  ]\n}"),
    ("block/diamond_emerald_chests.json", "{\n  \"values\": [\n    \"orechests:diamond_chest\",\n    \"orechests:emerald_chest\"\n  ]\n}"),
    ("block/diamond_emerald_obsidian_chests.json", "{\n  \"values\": [\n    \"orechests:diamond_chest\",\n    \"orechests:emerald_chest\",\n    \"orechests:obsidian_chest\"\n  ]\n}"),
];

pub fn ensure_content_and_recipes(root: &Path) {
    let mc_recipe_dir = root.join("data/minecraft/recipe");
    let ore_recipe_dir = root.join("data/orechests/recipe");
    let _ = fs::create_dir_all(&mc_recipe_dir);
    let _ = fs::create_dir_all(&ore_recipe_dir);

    for (name, content) in RECIPES {
        let mc_dest = mc_recipe_dir.join(format!("orechests_{name}"));
        let ore_dest = ore_recipe_dir.join(name);
        let _ = fs::write(&mc_dest, content);
        let _ = fs::write(&ore_dest, content);
    }

    for (rel_path, content) in TAGS {
        let tag_dest = root.join(format!("data/orechests/tags/{rel_path}"));
        if let Some(parent) = tag_dest.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(&tag_dest, content);
    }

    // Ensure orechests is listed in horizon-mod-content.json
    let marker_path = root.join("horizon-mod-content.json");
    let mut namespaces: Vec<String> = fs::read(&marker_path)
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default();
    if !namespaces.contains(&"orechests".to_string()) {
        namespaces.push("orechests".to_string());
        if let Ok(bytes) = serde_json::to_vec(&namespaces) {
            let _ = fs::write(marker_path, bytes);
        }
    }
}

pub fn auto_discover_root() -> Option<PathBuf> {
    if let Ok(exe) = std::env::current_exe() {
        let mut cur = exe.parent();
        while let Some(dir) = cur {
            if dir.join("data/minecraft/recipe").is_dir() {
                return Some(dir.to_path_buf());
            }
            if dir.join(".local/horizon-client/26.3").is_dir() {
                return Some(dir.join(".local/horizon-client/26.3"));
            }
            cur = dir.parent();
        }
    }
    None
}
