// Injected into horizon_client::game::native_hooks::Game<'_>::native_blocks

fn orechests_blocks(blocks: &mut Vec<horizon_mods::content::Block>) {
    crate::game::orechests_glue::register_blocks(blocks);
}
