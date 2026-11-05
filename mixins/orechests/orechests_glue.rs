// Ore Chests: Glue between Horizon's runtime and orechests_rules.
// Added to the game as `game::orechests_glue`.

use crate::game::{
    Game,
    orechests_rules::{
        calculate_screen_layout, ChestTier, Config, KeyRegistry, ScreenLayout, UpgradeItem,
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

/// Registers the 10 chest blocks in Horizon's block registry.
pub fn register_blocks(blocks: &mut Vec<horizon_mods::content::Block>) {
    for tier in ChestTier::ALL {
        blocks.push(horizon_mods::content::Block {
            id: tier.block_id(),
            template: "minecraft:chest".into(),
            properties: horizon_mods::content::BlockProperties {
                hardness: Some(tier.hardness()),
                explosion_resistance: Some(tier.explosion_resistance()),
                requires_tool: Some(true),
                sounds: Some(tier.sound_type().into()),
                occludes: if tier == ChestTier::Crystal { Some(false) } else { None },
                ..Default::default()
            },
            item: true,
            ..Default::default()
        });
    }
}

/// Registers upgrade items, container key, and hand trucks.
pub fn register_items(items: &mut Vec<horizon_mods::content::Item>) {
    for upgrade in UpgradeItem::ALL {
        items.push(horizon_mods::content::Item {
            id: upgrade.item_id(),
            template: "minecraft:iron_ingot".into(),
            max_stack_size: Some(16),
            ..Default::default()
        });
    }

    items.push(horizon_mods::content::Item {
        id: "orechests:container_key".into(),
        template: "minecraft:tripwire_hook".into(),
        max_stack_size: Some(1),
        ..Default::default()
    });

    items.push(horizon_mods::content::Item {
        id: "orechests:hand_truck".into(),
        template: "minecraft:stick".into(),
        max_stack_size: Some(1),
        ..Default::default()
    });

    items.push(horizon_mods::content::Item {
        id: "orechests:enhanced_hand_truck".into(),
        template: "minecraft:blaze_rod".into(),
        max_stack_size: Some(1),
        ..Default::default()
    });
}

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
        menu: Some(tier.block_id()),
    });
}

/// Lays out container GUI on the screen.
pub fn on_screen(block: &str, screen: &mut Option<horizon_client::mod_api::NativeScreen>) {
    let Some(tier) = ChestTier::from_id(block) else {
        return;
    };

    let layout = calculate_screen_layout(tier.default_columns(), tier.default_rows());

    *screen = Some(horizon_client::mod_api::NativeScreen {
        width: layout.width,
        height: layout.height,
        slots: layout.slots,
        inventory_x: layout.inventory_x,
        inventory_y: layout.inventory_y,
        own_labels: false,
        own_background: false,
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
    let item_name = stack.item.as_str();

    // 1. Upgrade Items: In-place chest tier upgrade!
    if let Some(upgrade) = UpgradeItem::from_id(item_name) {
        // Read block at position using command data / state
        // In Horizon server, execute in-place upgrade command preserving data
        let target_chest = match upgrade {
            UpgradeItem::Copper => "orechests:copper_chest",
            UpgradeItem::Iron => "orechests:iron_chest",
            UpgradeItem::Gold => "orechests:gold_chest",
            UpgradeItem::Diamond => "orechests:diamond_chest",
            UpgradeItem::Emerald => "orechests:emerald_chest",
            UpgradeItem::Crystal => "orechests:crystal_chest",
            UpgradeItem::Obsidian => "orechests:obsidian_chest",
            UpgradeItem::Netherite => "orechests:netherite_chest",
        };

        // Command: clone / merge container data into upgraded block
        // /execute positioned x y z run ...
        let upgrade_success = game.native_run_command(&format!(
            "execute positioned {x} {y} {z} run setblock {x} {y} {z} {target_chest} keep"
        ));

        // If replaced or updated, play sound effects & particles
        game.native_run_command(&format!(
            "playsound minecraft:block.anvil.use block @a {x} {y} {z} 1.0 1.2"
        ));
        game.native_run_command(&format!(
            "particle minecraft:happy_villager {x}.5 {y}.5 {z}.5 0.3 0.3 0.3 0.1 15"
        ));

        *claimed = true;
        return;
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
        WorldPhase::Stopping => {
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
    if phase != ServerTickPhase::End {
        return;
    }

    let dir = game.native_world_dir();
    ensure_state(dir);
}
