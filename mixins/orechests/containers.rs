// Injected into horizon_client::game::native_hooks::Game<'_> container and screen hooks

pub fn orechests_block_entity(
    block: &str,
    entity: &mut Option<(String, horizon_world::nbt::Compound)>,
) {
    crate::game::orechests_glue::on_block_entity(block, entity);
}

pub fn orechests_block_entity_container(
    block: &str,
    container: &mut Option<horizon_client::containers::ModContainer>,
) {
    crate::game::orechests_glue::on_block_entity_container(block, container);
}

pub fn orechests_screen(
    block: &str,
    screen: &mut Option<horizon_client::mod_api::NativeScreen>,
) {
    crate::game::orechests_glue::on_screen(block, screen);
}
