use rg3d::core::algebra::Vector3;
use rg3d::core::pool::Handle;
use rg3d::scene::camera::Camera;
use rg3d::scene::node::Node;
use rg3d::scene::Scene;

pub enum MoveDirection {
    Forward,
    Backward,
    Left,
    Right,
    Up,
    Down,
}

pub fn process_movement(direction: MoveDirection, scene: &mut Scene) {
    let handle = scene
        .graph
        .find_from_root(&mut |node| matches!(node, Node::Camera(_)));

    if handle != Handle::NONE {
        if let Node::Camera(camera) = &mut scene.graph[handle] {
            let speed = 50.0;

            let offset = match direction {
                MoveDirection::Forward => Vector3::new(0.0, 0.0, speed),
                MoveDirection::Backward => Vector3::new(0.0, 0.0, -speed),
                MoveDirection::Left => Vector3::new(speed, 0.0, 0.0),
                MoveDirection::Right => Vector3::new(-speed, 0.0, 0.0),
                MoveDirection::Up => Vector3::new(0.0, speed, 0.0),
                MoveDirection::Down => Vector3::new(0.0, -speed, 0.0),
            };

            camera.local_transform_mut().offset(offset);
        }
    }
}
