// Injected into horizon_client::game::native_hooks::Game<'_>::native_items

fn orechests_items(items: &mut Vec<horizon_mods::content::Item>) {
    crate::game::orechests_glue::register_items(items);
}
