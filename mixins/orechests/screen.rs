// Injected into horizon_client::game::native_hooks::Game<'_>::native_screen

fn orechests_screen(
    block: &str,
    screen: &mut Option<horizon_client::mod_api::NativeScreen>,
) {
    crate::game::orechests_glue::on_screen(block, screen);
}
