// Injected into horizon_client::game::load_world_rest

fn orechests_load_world_rest(root: &std::path::Path) {
    crate::game::orechests_glue::ensure_content_and_recipes(root);
}
