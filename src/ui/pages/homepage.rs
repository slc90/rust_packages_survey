use gpui::{Render, div};

pub struct MainPage {}

impl Render for MainPage {
    fn render(
        &mut self,
        _: &mut gpui::Window,
        _: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        div()
    }
}
