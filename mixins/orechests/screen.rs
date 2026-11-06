// Injected into horizon_client::game::native_hooks::Game<'_>::native_screen

fn orechests_screen(
    block: &str,
    screen: &mut Option<crate::game::native_hooks::NativeScreen>,
) {
    crate::game::orechests_glue::on_screen(block, screen);
}
