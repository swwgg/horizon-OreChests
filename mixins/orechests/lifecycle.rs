// Injected into horizon_client::game::native_hooks::Game<'_>::native_world_lifecycle

fn orechests_lifecycle(phase: crate::game::native_lifecycle::WorldPhase) {
    crate::game::orechests_glue::on_world_lifecycle(phase);
}
