// Ore Chests: Core game rules and data structures, independent of Horizon engine types.
// Unit tests and standalone library run without Horizon.

use serde::{Deserialize, Serialize};

/// Ore chest tiers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChestTier {
    Copper,
    Iron,
    Gold,
    Diamond,
    Netherite,
}

impl ChestTier {
    pub const ALL: [ChestTier; 5] = [
        ChestTier::Copper,
        ChestTier::Iron,
        ChestTier::Gold,
        ChestTier::Diamond,
        ChestTier::Netherite,
    ];

    pub const fn id(&self) -> &'static str {
        match self {
            ChestTier::Copper => "copper_chest",
            ChestTier::Iron => "iron_chest",
            ChestTier::Gold => "gold_chest",
            ChestTier::Diamond => "diamond_chest",
            ChestTier::Netherite => "netherite_chest",
        }
    }

    pub const fn block_id(&self) -> &'static str {
        match self {
            ChestTier::Copper => "orechests:copper_chest",
            ChestTier::Iron => "orechests:iron_chest",
            ChestTier::Gold => "orechests:gold_chest",
            ChestTier::Diamond => "orechests:diamond_chest",
            ChestTier::Netherite => "orechests:netherite_chest",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        let base = id.split('[').next().unwrap_or(id);
        let clean = base.strip_prefix("orechests:").unwrap_or(base);
        match clean {
            "copper_chest" => Some(ChestTier::Copper),
            "iron_chest" => Some(ChestTier::Iron),
            "gold_chest" => Some(ChestTier::Gold),
            "diamond_chest" => Some(ChestTier::Diamond),
            "netherite_chest" => Some(ChestTier::Netherite),
            _ => None,
        }
    }

    pub const fn default_columns(&self) -> usize {
        match self {
            ChestTier::Diamond => 12,
            ChestTier::Netherite => 14,
            _ => 9,
        }
    }

    pub const fn default_rows(&self) -> usize {
        match self {
            ChestTier::Copper => 5,
            ChestTier::Iron => 6,
            ChestTier::Gold | ChestTier::Diamond | ChestTier::Netherite => 9,
        }
    }

    pub const fn default_slots(&self) -> usize {
        self.default_columns() * self.default_rows()
    }

    pub const fn menu_id(&self) -> &'static str {
        match self {
            ChestTier::Copper => "minecraft:generic_9x5",
            ChestTier::Iron => "minecraft:generic_9x6",
            ChestTier::Gold => "orechests:gold_chest",
            ChestTier::Diamond => "orechests:diamond_chest",
            ChestTier::Netherite => "orechests:netherite_chest",
        }
    }

    /// Block hardness (destroy time in seconds).
    pub const fn hardness(&self) -> f32 {
        match self {
            ChestTier::Copper | ChestTier::Gold => 3.0,
            ChestTier::Iron | ChestTier::Diamond => 5.0,
            ChestTier::Netherite => 50.0,
        }
    }

    /// Explosion resistance.
    pub const fn explosion_resistance(&self) -> f32 {
        match self {
            ChestTier::Copper | ChestTier::Gold | ChestTier::Iron | ChestTier::Diamond => 6.0,
            ChestTier::Netherite => 1200.0,
        }
    }

    /// Vanilla sound template.
    pub const fn sound_type(&self) -> &'static str {
        match self {
            ChestTier::Copper | ChestTier::Gold => "minecraft:copper_block",
            ChestTier::Iron => "minecraft:iron_block",
            ChestTier::Diamond => "minecraft:stone",
            ChestTier::Netherite => "minecraft:netherite_block",
        }
    }

    /// Whether this chest is fireproof.
    pub const fn is_fireproof(&self) -> bool {
        matches!(self, ChestTier::Netherite)
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
        LEFT_INSET
    } else {
        slot_start_x + ((columns - VANILLA_COLUMNS) as f32 * SLOT_SIZE) / 2.0
    };

    // Header title takes 18px, then container rows, then 13px gap to inventory
    let inventory_y = 18.0 + (rows as f32 * SLOT_SIZE) + 13.0;
    // Vanilla inventory panel height is 96px (3 inventory rows + hotbar)
    let height = inventory_y + 96.0;

    let mut slots = Vec::with_capacity(columns * rows);
    for r in 0..rows {
        let y = 18.0 + (r as f32 * SLOT_SIZE);
        for c in 0..columns {
            let x = slot_start_x + (c as f32 * SLOT_SIZE);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tier_properties() {
        assert_eq!(ChestTier::Copper.default_slots(), 45);
        assert_eq!(ChestTier::Iron.default_slots(), 54);
        assert_eq!(ChestTier::Gold.default_slots(), 81);
        assert_eq!(ChestTier::Diamond.default_slots(), 108);
        assert_eq!(ChestTier::Netherite.default_slots(), 126);

        // Strict progression check
        assert!(ChestTier::Copper.default_slots() < ChestTier::Iron.default_slots());
        assert!(ChestTier::Iron.default_slots() < ChestTier::Gold.default_slots());
        assert!(ChestTier::Gold.default_slots() < ChestTier::Diamond.default_slots());
        assert!(ChestTier::Diamond.default_slots() < ChestTier::Netherite.default_slots());

        assert_eq!(ChestTier::Copper.menu_id(), "minecraft:generic_9x5");
        assert_eq!(ChestTier::Iron.menu_id(), "minecraft:generic_9x6");
        assert_eq!(ChestTier::Gold.menu_id(), "orechests:gold_chest");
        assert_eq!(ChestTier::Diamond.menu_id(), "orechests:diamond_chest");
        assert_eq!(ChestTier::Netherite.menu_id(), "orechests:netherite_chest");

        assert!(ChestTier::Netherite.is_fireproof());
        assert!(!ChestTier::Iron.is_fireproof());
    }

    #[test]
    fn test_from_id_namespace_and_blockstates() {
        assert_eq!(ChestTier::from_id("orechests:copper_chest"), Some(ChestTier::Copper));
        assert_eq!(ChestTier::from_id("copper_chest"), Some(ChestTier::Copper));
        assert_eq!(
            ChestTier::from_id("orechests:diamond_chest[facing=north]"),
            Some(ChestTier::Diamond)
        );
        // Must not hijack vanilla copper_chest or removed dirt_chest
        assert_eq!(ChestTier::from_id("minecraft:copper_chest"), None);
        assert_eq!(ChestTier::from_id("orechests:dirt_chest"), None);
    }

    #[test]
    fn test_screen_layouts() {
        let l_copper = calculate_screen_layout(9, 5);
        assert_eq!(l_copper.width, 176.0);
        assert_eq!(l_copper.slots.len(), 45);
        assert_eq!(l_copper.inventory_x, 8.0);
        assert_eq!(l_copper.inventory_y, 121.0);
        assert_eq!(l_copper.height, 217.0);

        let l_diamond = calculate_screen_layout(12, 9);
        assert_eq!(l_diamond.width, 231.0);
        assert_eq!(l_diamond.slots.len(), 108);
        assert_eq!(l_diamond.inventory_x, 35.0);
        assert_eq!(l_diamond.inventory_y, 193.0);
        assert_eq!(l_diamond.height, 289.0);
    }
}
