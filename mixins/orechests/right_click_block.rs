// Injected into horizon_client::game::native_hooks::Game<'_>::native_right_click_block

fn orechests_right_click_block(
    game: &mut crate::game::Game<'_>,
    pos: [i32; 3],
    face: [i32; 3],
    blocked: &mut bool,
) {
    crate::game::orechests_glue::on_right_click_block(game, pos, face, blocked);
}
