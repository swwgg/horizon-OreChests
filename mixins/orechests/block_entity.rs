// Injected into horizon_client::game::native_hooks::Game<'_>::native_block_entity

fn orechests_block_entity(
    block: &str,
    entity: &mut Option<(String, horizon_world::nbt::Compound)>,
) {
    crate::game::orechests_glue::on_block_entity(block, entity);
}
