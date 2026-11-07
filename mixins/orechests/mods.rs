// Injected into horizon_client::game::native_hooks::Game<'_>::native_mods

fn orechests_mods(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    mods.push(crate::game::native_hooks::NativeMod {
        id: "orechests".into(),
        title: "Ore Chests 1.0.0".into(),
        description: "Tiered ore chests: Copper, Iron, Gold, Diamond, Netherite.".into(),
        keys: vec![],
    });
}
