use rg3d::core::color::Color;
use rg3d::core::pool::Handle;
use rg3d::engine::framework::prelude::*;
use rg3d::gui::message::{MessageDirection, TextMessage};
use rg3d::gui::node::{StubNode, UINode};
use rg3d::gui::text::TextBuilder;
use rg3d::gui::widget::WidgetBuilder;

mod loader;

type BuildContext<'a> = rg3d::gui::BuildContext<'a, (), StubNode>;

fn create_ui(ctx: &mut BuildContext) -> Handle<UiNode> {
    TextBuilder::new(WidgetBuilder::new()).build(ctx)
}

struct Game {
    debug_text: Handle<UINode<(), StubNode>>,
}

impl GameState for Game {
    fn init(engine: &mut GameEngine) -> Self
    where
        Self: Sized,
    {
        let debug_text = create_ui(&mut engine.user_interface.build_ctx());

        // Set clear color
        engine
            .renderer
            .set_backbuffer_clear_color(Color::from_rgba(0x66, 0x66, 0x66, 255));

        Self { debug_text }
    }

    // Implement a function that will update game logic and will be called at fixed rate of 60 Hz.
    fn on_tick(&mut self, engine: &mut GameEngine, _dt: f32) {
        let fps = engine.renderer.get_statistics().frames_per_second;
        let text = format!("FPS: {}", fps);
        engine.user_interface.send_message(TextMessage::text(
            self.debug_text,
            MessageDirection::ToWidget,
            text,
        ));
    }
}

fn main() {
    // Framework is a simple wrapper that initializes engine and hides game loop details, allowing
    // you to focus only on important things.
    Framework::<Game>::new().unwrap().title("Bsp-rs").run();
}
