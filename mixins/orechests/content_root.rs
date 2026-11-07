// Injected into horizon_client::game::mods::content_root

fn orechests_content_root(root: &std::path::Path) {
    crate::game::orechests_glue::ensure_content_mod_installed();
    crate::game::orechests_glue::ensure_content_and_recipes(root);
}
