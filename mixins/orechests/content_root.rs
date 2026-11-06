// Injected into horizon_client::game::mods::content_root

fn orechests_content_root() {
    crate::game::orechests_glue::ensure_content_mod_installed();
}
