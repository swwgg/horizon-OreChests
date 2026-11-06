// Injected into horizon_client::game::native_hooks::Game<'_>::native_use_item_on

fn orechests_use_item_on(
    game: &mut crate::game::Game<'_>,
    stack: &horizon_client::items::ItemStack,
    pos: [i32; 3],
    face: [i32; 3],
    blocked: &mut bool,
) {
    crate::game::orechests_glue::on_use_item_on(game, stack, pos, face, blocked);
}
