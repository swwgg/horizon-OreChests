// Ore Chests: Core game rules and data structures, independent of Horizon engine types.
// Unit tests and standalone library run without Horizon.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};

/// Chest tiers ported from IronChests.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChestTier {
    Dirt,
    Copper,
    Iron,
    Gold,
    Diamond,
    Netherite,
}

impl ChestTier {
    pub const ALL: [ChestTier; 6] = [
        ChestTier::Dirt,
        ChestTier::Copper,
        ChestTier::Iron,
        ChestTier::Gold,
        ChestTier::Diamond,
        ChestTier::Netherite,
    ];

    pub fn id(&self) -> &'static str {
        match self {
            ChestTier::Dirt => "dirt_chest",
            ChestTier::Copper => "copper_chest",
            ChestTier::Iron => "iron_chest",
            ChestTier::Gold => "gold_chest",
            ChestTier::Diamond => "diamond_chest",
            ChestTier::Netherite => "netherite_chest",
        }
    }

    pub fn block_id(&self) -> String {
        format!("orechests:{}", self.id())
    }

    pub fn from_id(id: &str) -> Option<Self> {
        let clean = id
            .strip_prefix("orechests:")
            .or_else(|| id.strip_prefix("ironchest:"))
            .or_else(|| id.strip_prefix("minecraft:"))
            .unwrap_or(id);
        match clean {
            "dirt_chest" => Some(ChestTier::Dirt),
            "copper_chest" => Some(ChestTier::Copper),
            "iron_chest" => Some(ChestTier::Iron),
            "gold_chest" => Some(ChestTier::Gold),
            "diamond_chest" => Some(ChestTier::Diamond),
            "netherite_chest" => Some(ChestTier::Netherite),
            _ => None,
        }
    }

    pub fn default_columns(&self) -> usize {
        match self {
            ChestTier::Diamond => 12,
            ChestTier::Netherite => 14,
            _ => 9,
        }
    }

    pub fn default_rows(&self) -> usize {
        match self {
            ChestTier::Dirt => 1,
            ChestTier::Copper => 5,
            ChestTier::Iron => 6,
            ChestTier::Gold => 9,
            ChestTier::Diamond => 9,
            ChestTier::Netherite => 9,
        }
    }

    pub fn default_slots(&self) -> usize {
        self.default_columns() * self.default_rows()
    }

    pub fn menu_id(&self) -> &'static str {
        match self {
            ChestTier::Dirt => "minecraft:generic_9x1",
            ChestTier::Copper => "minecraft:generic_9x5",
            ChestTier::Iron => "minecraft:generic_9x6",
            ChestTier::Gold => "orechests:gold_chest",
            ChestTier::Diamond => "orechests:diamond_chest",
            ChestTier::Netherite => "orechests:netherite_chest",
        }
    }

    /// Block hardness (destroy time in seconds)
    pub fn hardness(&self) -> f32 {
        match self {
            ChestTier::Dirt => 0.5,
            ChestTier::Copper | ChestTier::Gold => 3.0,
            ChestTier::Iron => 5.0,
            ChestTier::Diamond => 5.0,
            ChestTier::Netherite => 50.0,
        }
    }

    /// Explosion resistance
    pub fn explosion_resistance(&self) -> f32 {
        match self {
            ChestTier::Dirt => 0.5,
            ChestTier::Copper | ChestTier::Gold | ChestTier::Iron | ChestTier::Diamond => 6.0,
            ChestTier::Netherite => 1200.0,
        }
    }

    /// Vanilla sound template
    pub fn sound_type(&self) -> &'static str {
        match self {
            ChestTier::Dirt => "minecraft:gravel",
            ChestTier::Copper => "minecraft:copper_block",
            ChestTier::Iron => "minecraft:iron_block",
            ChestTier::Gold => "minecraft:copper_block",
            ChestTier::Diamond => "minecraft:stone",
            ChestTier::Netherite => "minecraft:netherite_block",
        }
    }

    /// Whether this chest is fireproof
    pub fn is_fireproof(&self) -> bool {
        matches!(self, ChestTier::Netherite)
    }

    /// Whether this chest only accepts dirt items
    pub fn is_dirt_only(&self) -> bool {
        matches!(self, ChestTier::Dirt)
    }
}

/// Upgrade items that transform chests in place.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpgradeItem {
    Copper,
    Iron,
    Gold,
    Diamond,
    Netherite,
}

impl UpgradeItem {
    pub const ALL: [UpgradeItem; 5] = [
        UpgradeItem::Copper,
        UpgradeItem::Iron,
        UpgradeItem::Gold,
        UpgradeItem::Diamond,
        UpgradeItem::Netherite,
    ];

    pub fn id(&self) -> &'static str {
        match self {
            UpgradeItem::Copper => "copper_upgrade",
            UpgradeItem::Iron => "iron_upgrade",
            UpgradeItem::Gold => "gold_upgrade",
            UpgradeItem::Diamond => "diamond_upgrade",
            UpgradeItem::Netherite => "netherite_upgrade",
        }
    }

    pub fn item_id(&self) -> String {
        format!("orechests:{}", self.id())
    }

    pub fn from_id(id: &str) -> Option<Self> {
        let clean = id.strip_prefix("orechests:").or_else(|| id.strip_prefix("ironcore:")).unwrap_or(id);
        match clean {
            "copper_upgrade" => Some(UpgradeItem::Copper),
            "iron_upgrade" => Some(UpgradeItem::Iron),
            "gold_upgrade" => Some(UpgradeItem::Gold),
            "diamond_upgrade" => Some(UpgradeItem::Diamond),
            "netherite_upgrade" => Some(UpgradeItem::Netherite),
            _ => None,
        }
    }

    /// Checks if this upgrade can be applied to `source_block`.
    /// `source_block` can be vanilla "minecraft:chest", "minecraft:copper_chest" or an ore chest.
    pub fn target_tier_from_block(&self, source_block: &str) -> Option<ChestTier> {
        let clean = source_block
            .strip_prefix("orechests:")
            .or_else(|| source_block.strip_prefix("ironchest:"))
            .or_else(|| source_block.strip_prefix("minecraft:"))
            .unwrap_or(source_block);
        let is_vanilla_wood = clean == "chest" || clean == "trapped_chest" || clean == "barrel";

        match self {
            UpgradeItem::Copper => {
                if is_vanilla_wood {
                    Some(ChestTier::Copper)
                } else {
                    None
                }
            }
            UpgradeItem::Iron => {
                if clean == "copper_chest" {
                    Some(ChestTier::Iron)
                } else {
                    None
                }
            }
            UpgradeItem::Gold => {
                if clean == "iron_chest" {
                    Some(ChestTier::Gold)
                } else {
                    None
                }
            }
            UpgradeItem::Diamond => {
                if clean == "gold_chest" {
                    Some(ChestTier::Diamond)
                } else {
                    None
                }
            }
            UpgradeItem::Netherite => {
                if clean == "diamond_chest" {
                    Some(ChestTier::Netherite)
                } else {
                    None
                }
            }
        }
    }
}

/// Calculated screen geometry for a chest container.
#[derive(Clone, Debug, PartialEq)]
pub struct ScreenLayout {
    pub width: f32,
    pub height: f32,
    pub inventory_x: f32,
    pub inventory_y: f32,
    pub slots: Vec<Option<[f32; 2]>>,
}

/// Calculates exact slot positions and player inventory layout based on columns and rows.
pub fn calculate_screen_layout(columns: usize, rows: usize) -> ScreenLayout {
    const SLOT_SIZE: f32 = 18.0;
    const LEFT_INSET: f32 = 8.0;
    const RIGHT_INSET: f32 = 7.0;
    const VANILLA_COLUMNS: usize = 9;
    const VANILLA_WIDTH: f32 = 176.0;

    let width = if columns <= VANILLA_COLUMNS {
        VANILLA_WIDTH
    } else {
        LEFT_INSET + (columns as f32 * SLOT_SIZE) + RIGHT_INSET
    };

    let slot_start_x = LEFT_INSET;
    let inventory_x = if columns <= VANILLA_COLUMNS {
        8.0
    } else {
        slot_start_x + ((columns as f32 * SLOT_SIZE) - (VANILLA_COLUMNS as f32 * SLOT_SIZE)) / 2.0
    };

    // Header title takes 18px, then container rows, then 13px gap to inventory
    let inventory_y = 18.0 + (rows as f32 * SLOT_SIZE) + 13.0;
    // Vanilla inventory panel height is 96px (3 inventory rows + hotbar)
    let height = inventory_y + 96.0;

    let mut slots = Vec::with_capacity(columns * rows);
    for r in 0..rows {
        for c in 0..columns {
            let x = slot_start_x + (c as f32 * SLOT_SIZE);
            let y = 18.0 + (r as f32 * SLOT_SIZE);
            slots.push(Some([x, y]));
        }
    }

    ScreenLayout {
        width,
        height,
        inventory_x,
        inventory_y,
        slots,
    }
}

/// Key registry for container locking.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct KeyRegistry {
    pub keys: HashMap<u128, KeyEntry>,
    pub locked_blocks: HashMap<[i32; 3], u128>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KeyEntry {
    pub owner_uuid: u128,
    pub label: String,
    pub code: u32,
    pub trusted_uuids: HashSet<u128>,
}

impl KeyRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn lock(&mut self, pos: [i32; 3], key_id: u128, owner_uuid: u128, label: &str, code: u32) {
        self.locked_blocks.insert(pos, key_id);
        self.keys.entry(key_id).or_insert_with(|| KeyEntry {
            owner_uuid,
            label: label.to_owned(),
            code,
            trusted_uuids: HashSet::new(),
        });
    }

    pub fn unlock(&mut self, pos: [i32; 3]) -> Option<u128> {
        self.locked_blocks.remove(&pos)
    }

    pub fn is_locked(&self, pos: [i32; 3]) -> bool {
        self.locked_blocks.contains_key(&pos)
    }

    pub fn can_open(&self, pos: [i32; 3], player_uuid: u128) -> bool {
        let Some(&key_id) = self.locked_blocks.get(&pos) else {
            return true; // Not locked
        };
        let Some(entry) = self.keys.get(&key_id) else {
            return true; // Orphaned lock
        };
        entry.owner_uuid == player_uuid || entry.trusted_uuids.contains(&player_uuid)
    }

    pub fn add_trusted(&mut self, key_id: u128, trusted_uuid: u128) -> bool {
        if let Some(entry) = self.keys.get_mut(&key_id) {
            entry.trusted_uuids.insert(trusted_uuid)
        } else {
            false
        }
    }
}

/// Carried container representation for hand truck.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CarriedContainer {
    pub block_id: String,
    pub facing: String,
    pub items: Vec<Option<CarriedItem>>,
    pub is_spawner: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CarriedItem {
    pub slot: usize,
    pub item: String,
    pub count: u32,
    pub components: Option<String>,
}

/// Mod configuration.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub open_under_solid_blocks: bool,
    pub copper_golem_interaction: bool,
    pub locks_enabled: bool,
    pub moving_spawners_enabled: bool,
    pub custom_rows: HashMap<String, usize>,
}

impl Default for Config {
    fn default() -> Self {
        let mut custom_rows = HashMap::new();
        for tier in ChestTier::ALL {
            custom_rows.insert(tier.id().to_owned(), tier.default_rows());
        }
        Self {
            open_under_solid_blocks: false,
            copper_golem_interaction: true,
            locks_enabled: true,
            moving_spawners_enabled: true,
            custom_rows,
        }
    }
}

impl Config {
    pub fn to_json(&self) -> Value {
        json!({
            "open_under_solid_blocks": self.open_under_solid_blocks,
            "copper_golem_interaction": self.copper_golem_interaction,
            "locks_enabled": self.locks_enabled,
            "moving_spawners_enabled": self.moving_spawners_enabled,
            "rows": self.custom_rows,
        })
    }

    pub fn from_json(val: &Value) -> Self {
        let mut cfg = Self::default();
        if let Some(b) = val.get("open_under_solid_blocks").and_then(Value::as_bool) {
            cfg.open_under_solid_blocks = b;
        }
        if let Some(b) = val.get("copper_golem_interaction").and_then(Value::as_bool) {
            cfg.copper_golem_interaction = b;
        }
        if let Some(b) = val.get("locks_enabled").and_then(Value::as_bool) {
            cfg.locks_enabled = b;
        }
        if let Some(b) = val.get("moving_spawners_enabled").and_then(Value::as_bool) {
            cfg.moving_spawners_enabled = b;
        }
        if let Some(rows) = val.get("rows").and_then(Value::as_object) {
            for (k, v) in rows {
                if let Some(n) = v.as_u64() {
                    cfg.custom_rows.insert(k.clone(), (n as usize).clamp(1, 12));
                }
            }
        }
        cfg
    }

    pub fn get_rows(&self, tier: ChestTier) -> usize {
        self.custom_rows.get(tier.id()).copied().unwrap_or_else(|| tier.default_rows())
    }

    pub fn get_slots(&self, tier: ChestTier) -> usize {
        tier.default_columns() * self.get_rows(tier)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tier_properties() {
        assert_eq!(ChestTier::Dirt.default_slots(), 9);
        assert_eq!(ChestTier::Copper.default_slots(), 45);
        assert_eq!(ChestTier::Iron.default_slots(), 54);
        assert_eq!(ChestTier::Gold.default_slots(), 81);
        assert_eq!(ChestTier::Diamond.default_slots(), 108);
        assert_eq!(ChestTier::Netherite.default_slots(), 126);

        // Strict progression check
        assert!(ChestTier::Dirt.default_slots() < ChestTier::Copper.default_slots());
        assert!(ChestTier::Copper.default_slots() < ChestTier::Iron.default_slots());
        assert!(ChestTier::Iron.default_slots() < ChestTier::Gold.default_slots());
        assert!(ChestTier::Gold.default_slots() < ChestTier::Diamond.default_slots());
        assert!(ChestTier::Diamond.default_slots() < ChestTier::Netherite.default_slots());

        assert_eq!(ChestTier::Dirt.menu_id(), "minecraft:generic_9x1");
        assert_eq!(ChestTier::Copper.menu_id(), "minecraft:generic_9x5");
        assert_eq!(ChestTier::Iron.menu_id(), "minecraft:generic_9x6");
        assert_eq!(ChestTier::Gold.menu_id(), "orechests:gold_chest");
        assert_eq!(ChestTier::Diamond.menu_id(), "orechests:diamond_chest");
        assert_eq!(ChestTier::Netherite.menu_id(), "orechests:netherite_chest");

        assert!(ChestTier::Netherite.is_fireproof());
        assert!(!ChestTier::Iron.is_fireproof());
        assert!(ChestTier::Dirt.is_dirt_only());
        assert!(!ChestTier::Copper.is_dirt_only());
    }

    #[test]
    fn test_upgrade_ladder() {
        // Vanilla wood chest -> copper
        assert_eq!(
            UpgradeItem::Copper.target_tier_from_block("minecraft:chest"),
            Some(ChestTier::Copper)
        );
        // Copper -> Iron
        assert_eq!(
            UpgradeItem::Iron.target_tier_from_block("orechests:copper_chest"),
            Some(ChestTier::Iron)
        );
        assert_eq!(
            UpgradeItem::Iron.target_tier_from_block("minecraft:copper_chest"),
            Some(ChestTier::Iron)
        );
        // Iron -> Gold
        assert_eq!(
            UpgradeItem::Gold.target_tier_from_block("iron_chest"),
            Some(ChestTier::Gold)
        );
        // Gold -> Diamond
        assert_eq!(
            UpgradeItem::Diamond.target_tier_from_block("gold_chest"),
            Some(ChestTier::Diamond)
        );
        // Diamond -> Netherite
        assert_eq!(
            UpgradeItem::Netherite.target_tier_from_block("diamond_chest"),
            Some(ChestTier::Netherite)
        );

        // Invalid transitions
        assert_eq!(UpgradeItem::Gold.target_tier_from_block("copper_chest"), None);
        assert_eq!(UpgradeItem::Copper.target_tier_from_block("iron_chest"), None);
        assert_eq!(UpgradeItem::Diamond.target_tier_from_block("dirt_chest"), None);
    }

    #[test]
    fn test_screen_layouts() {
        let l_copper = calculate_screen_layout(9, 5);
        assert_eq!(l_copper.width, 176.0);
        assert_eq!(l_copper.slots.len(), 45);
        assert_eq!(l_copper.inventory_x, 8.0);
        assert_eq!(l_copper.inventory_y, 18.0 + 5.0 * 18.0 + 13.0); // 121.0
        assert_eq!(l_copper.height, 121.0 + 96.0); // 217.0

        let l_iron = calculate_screen_layout(9, 6);
        assert_eq!(l_iron.width, 176.0);
        assert_eq!(l_iron.slots.len(), 54);
        assert_eq!(l_iron.inventory_x, 8.0);
        assert_eq!(l_iron.inventory_y, 18.0 + 6.0 * 18.0 + 13.0); // 139.0
        assert_eq!(l_iron.height, 139.0 + 96.0); // 235.0
    }

    #[test]
    fn test_key_registry() {
        let mut reg = KeyRegistry::new();
        let pos = [100, 64, -200];
        let key_id = 0x1234_5678_9abc_def0;
        let owner = 0x1111;
        let stranger = 0x2222;
        let friend = 0x3333;

        reg.lock(pos, key_id, owner, "My Safe", 1001);
        assert!(reg.is_locked(pos));
        assert!(reg.can_open(pos, owner));
        assert!(!reg.can_open(pos, stranger));

        reg.add_trusted(key_id, friend);
        assert!(reg.can_open(pos, friend));

        reg.unlock(pos);
        assert!(!reg.is_locked(pos));
        assert!(reg.can_open(pos, stranger));
    }

    #[test]
    fn test_config_serde() {
        let mut cfg = Config::default();
        cfg.open_under_solid_blocks = true;
        cfg.custom_rows.insert("copper_chest".into(), 4);

        let json = cfg.to_json();
        let loaded = Config::from_json(&json);

        assert_eq!(loaded.open_under_solid_blocks, true);
        assert_eq!(loaded.get_rows(ChestTier::Copper), 4);
        assert_eq!(loaded.get_slots(ChestTier::Copper), 36);
    }
}
