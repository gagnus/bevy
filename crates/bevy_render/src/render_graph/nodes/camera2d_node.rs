use bevy_app::Events;
use bevy_window::WindowResized;

use crate::{
    camera::{ActiveCamera2d, Camera},
    render_graph::{CommandQueue, Node, ResourceSlots, SystemNode},
    render_resource::{
        resource_name, BufferInfo, BufferUsage, RenderResourceAssignment, RenderResourceAssignments,
    },
    renderer::{RenderContext, RenderResources},
};

use legion::prelude::*;
use zerocopy::AsBytes;

#[derive(Default)]
pub struct Camera2dNode {
    command_queue: CommandQueue,
}

impl Node for Camera2dNode {
    fn update(
        &mut self,
        _world: &World,
        _resources: &Resources,
        render_context: &mut dyn RenderContext,
        _input: &ResourceSlots,
        _output: &mut ResourceSlots,
    ) {
        self.command_queue.execute(render_context);
    }
}

impl SystemNode for Camera2dNode {
    fn get_system(&self) -> Box<dyn Schedulable> {
        let mut camera_buffer = None;
        let mut window_resized_event_reader = None;
        let mut command_queue = self.command_queue.clone();

        SystemBuilder::new("camera_2d_resource_provider")
            .read_resource::<RenderResources>()
            // TODO: this write on RenderResourceAssignments will prevent this system from running in parallel with other systems that do the same
            .write_resource::<RenderResourceAssignments>()
            .read_resource::<Events<WindowResized>>()
            .with_query(<(Read<Camera>, Read<ActiveCamera2d>)>::query())
            .build(
                move |_,
                      world,
                      (
                    render_resource_context,
                    ref mut render_resource_assignments,
                    window_resized_events,
                ),
                      query| {
                    let render_resources = &render_resource_context.context;
                    if camera_buffer.is_none() {
                        let size = std::mem::size_of::<[[f32; 4]; 4]>();
                        let buffer = render_resources.create_buffer(BufferInfo {
                            size,
                            buffer_usage: BufferUsage::COPY_DST | BufferUsage::UNIFORM,
                            ..Default::default()
                        });
                        render_resource_assignments.set(
                            resource_name::uniform::CAMERA2D,
                            RenderResourceAssignment::Buffer {
                                resource: buffer,
                                range: 0..size as u64,
                                dynamic_index: None,
                            },
                        );
                        camera_buffer = Some(buffer);
                    }

                    if window_resized_event_reader.is_none() {
                        window_resized_event_reader = Some(window_resized_events.get_reader());
                    }
                    let primary_window_resized_event = window_resized_event_reader
                        .as_mut()
                        .unwrap()
                        .find_latest(&window_resized_events, |event| event.is_primary);
                    if let Some(_) = primary_window_resized_event {
                        let matrix_size = std::mem::size_of::<[[f32; 4]; 4]>();
                        for (camera, _) in query.iter(world) {
                            let camera_matrix: [[f32; 4]; 4] =
                                camera.view_matrix.to_cols_array_2d();

                            let tmp_buffer = render_resources.create_buffer_mapped(
                                BufferInfo {
                                    size: matrix_size,
                                    buffer_usage: BufferUsage::COPY_SRC,
                                    ..Default::default()
                                },
                                &mut |data, _renderer| {
                                    data[0..matrix_size].copy_from_slice(camera_matrix.as_bytes());
                                },
                            );

                            command_queue.copy_buffer_to_buffer(
                                tmp_buffer,
                                0,
                                camera_buffer.unwrap(),
                                0,
                                matrix_size as u64,
                            );
                            command_queue.free_buffer(tmp_buffer);
                        }
                    }
                },
            )
    }
}
