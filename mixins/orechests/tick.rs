// Injected into horizon_client::game::native_hooks::Game<'_>::native_server_tick

fn orechests_tick(
    game: &mut crate::game::Game<'_>,
    phase: crate::game::native_lifecycle::ServerTickPhase,
) {
    crate::game::orechests_glue::on_server_tick(game, phase);
}
