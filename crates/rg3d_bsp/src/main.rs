use rg3d::core::color::Color;
use rg3d::core::pool::Handle;
use rg3d::engine::framework::prelude::*;
use rg3d::event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent};
use rg3d::gui::message::{MessageDirection, TextMessage};
use rg3d::gui::node::{StubNode, UINode};
use rg3d::gui::text::TextBuilder;
use rg3d::gui::widget::WidgetBuilder;
use rg3d::renderer::QualitySettings;
use rg3d::scene::Scene;

use self::input::MoveDirection;

mod input;
mod loader;

type BuildContext<'a> = rg3d::gui::BuildContext<'a, (), StubNode>;

fn create_ui(ctx: &mut BuildContext) -> Handle<UiNode> {
    TextBuilder::new(WidgetBuilder::new()).build(ctx)
}

struct Game {
    debug_text: Handle<UINode<(), StubNode>>,
    scene_handles: Vec<Handle<Scene>>,
    current_scene: usize,
}

impl Game {
    fn current_scene_mut<'a>(&mut self, engine: &'a mut GameEngine) -> Option<&'a mut Scene> {
        let handle = self.scene_handles.get(self.current_scene)?;

        Some(&mut engine.scenes[*handle])
    }
}

impl GameState for Game {
    fn init(engine: &mut GameEngine) -> Self
    where
        Self: Sized,
    {
        let settings = QualitySettings::ultra();
        engine.renderer.set_quality_settings(&settings).unwrap();

        let debug_text = create_ui(&mut engine.user_interface.build_ctx());

        let wad_manager = loader::WadManager::new("assets/wads");

        let maps = loader::load_maps("assets/maps", &wad_manager);

        let mut scene_handles = vec![];

        for (idx, mut scene) in maps.into_iter().enumerate() {
            if idx == 0 {
                scene.enabled = true;
            }

            scene_handles.push(engine.scenes.add(scene));
        }

        Self {
            debug_text,
            scene_handles,
            current_scene: 0,
        }
    }

    // Implement a function that will update game logic and will be called at fixed rate of 60 Hz.
    fn on_tick(&mut self, engine: &mut GameEngine, _dt: f32) {
        // Set clear color
        engine
            .renderer
            .set_backbuffer_clear_color(Color::from_rgba(0x66, 0x66, 0x66, 255));

        let fps = engine.renderer.get_statistics().frames_per_second;
        let text = format!("FPS: {}", fps);
        engine.user_interface.send_message(TextMessage::text(
            self.debug_text,
            MessageDirection::ToWidget,
            text,
        ));
    }

    fn on_window_event(&mut self, engine: &mut GameEngine, event: WindowEvent) {
        match event {
            WindowEvent::KeyboardInput { input, .. } => {
                if let Some(key_code) = input.virtual_keycode {
                    match key_code {
                        VirtualKeyCode::Left if input.state == ElementState::Released => {
                            let handle = self.scene_handles[self.current_scene];
                            engine.scenes[handle].enabled = false;

                            self.current_scene = self
                                .current_scene
                                .wrapping_sub(1)
                                .min(self.scene_handles.len() - 1);

                            let handle = self.scene_handles[self.current_scene];
                            engine.scenes[handle].enabled = true;
                        }
                        VirtualKeyCode::Right if input.state == ElementState::Released => {
                            let handle = self.scene_handles[self.current_scene];
                            engine.scenes[handle].enabled = false;

                            self.current_scene =
                                (self.current_scene + 1) % self.scene_handles.len();

                            let handle = self.scene_handles[self.current_scene];
                            engine.scenes[handle].enabled = true;
                        }
                        VirtualKeyCode::W => {
                            if let Some(scene) = self.current_scene_mut(engine) {
                                input::process_movement(MoveDirection::Forward, scene);
                            }
                        }
                        VirtualKeyCode::A => {
                            if let Some(scene) = self.current_scene_mut(engine) {
                                input::process_movement(MoveDirection::Left, scene);
                            }
                        }
                        VirtualKeyCode::S => {
                            if let Some(scene) = self.current_scene_mut(engine) {
                                input::process_movement(MoveDirection::Backward, scene);
                            }
                        }
                        VirtualKeyCode::D => {
                            if let Some(scene) = self.current_scene_mut(engine) {
                                input::process_movement(MoveDirection::Right, scene);
                            }
                        }
                        VirtualKeyCode::LShift => {
                            if let Some(scene) = self.current_scene_mut(engine) {
                                input::process_movement(MoveDirection::Down, scene);
                            }
                        }
                        VirtualKeyCode::Space => {
                            if let Some(scene) = self.current_scene_mut(engine) {
                                input::process_movement(MoveDirection::Up, scene);
                            }
                        }
                        _ => (),
                    }
                }
            }
            WindowEvent::AxisMotion { axis, value, .. } => {
                // asdf
            }
            _ => {}
        }
    }
}

fn main() {
    // Framework is a simple wrapper that initializes engine and hides game loop details, allowing
    // you to focus only on important things.
    Framework::<Game>::new().unwrap().title("Bsp-rs").run();
}
