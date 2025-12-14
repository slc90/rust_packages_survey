use gpui::{AppContext, Application, WindowOptions};
use gpui_component::Root;

mod ui;

use crate::ui::pages::homepage::MainPage;

/// 程序的主入口
fn main() {
    let app = Application::new();
    app.run(move |cx| {
        // This must be called before using any GPUI Component features.
        gpui_component::init(cx);

        cx.spawn(async move |cx| {
            cx.open_window(WindowOptions::default(), |window, cx| {
                let view = cx.new(|_| MainPage {});
                // This first level on the window, should be a Root.
                cx.new(|cx| Root::new(view, window, cx))
            })?;

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
}
