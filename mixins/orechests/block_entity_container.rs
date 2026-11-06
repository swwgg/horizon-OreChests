// Injected into horizon_client::game::native_hooks::Game<'_>::native_block_entity_container

fn orechests_block_entity_container(
    block: &str,
    container: &mut Option<horizon_client::containers::ModContainer>,
) {
    crate::game::orechests_glue::on_block_entity_container(block, container);
}
