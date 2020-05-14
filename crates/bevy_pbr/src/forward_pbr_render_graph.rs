use crate::{material::StandardMaterial, nodes::LightsNode, pipelines::build_forward_pipeline};
use bevy_asset::Assets;
use bevy_render::{
    base_render_graph,
    draw_target::AssignedMeshesDrawTarget,
    pipeline::PipelineDescriptor,
    render_graph::{
        nodes::{AssetUniformNode, PassNode, UniformNode},
        RenderGraph,
    },
    shader::Shader,
};
use bevy_transform::prelude::LocalToWorld;
use legion::prelude::Resources;

pub mod node {
    pub const LOCAL_TO_WORLD: &str = "local_to_world";
    pub const STANDARD_MATERIAL: &str = "standard_material";
    pub const LIGHTS: &str = "lights";
}

pub trait ForwardPbrRenderGraphBuilder {
    fn add_pbr_graph(&mut self, resources: &Resources) -> &mut Self;
}

impl ForwardPbrRenderGraphBuilder for RenderGraph {
    fn add_pbr_graph(&mut self, resources: &Resources) -> &mut Self {
        self.add_system_node_named(
            node::LOCAL_TO_WORLD,
            UniformNode::<LocalToWorld>::new(true)
        );
        self.add_system_node_named(
            node::STANDARD_MATERIAL,
            AssetUniformNode::<StandardMaterial>::new(true)
        );
        self.add_system_node_named(node::LIGHTS, LightsNode::new(10));
        let mut shaders = resources.get_mut::<Assets<Shader>>().unwrap();
        let mut pipelines = resources
            .get_mut::<Assets<PipelineDescriptor>>()
            .unwrap();
        {
            let main_pass: &mut PassNode = self
                .get_node_mut(base_render_graph::node::MAIN_PASS)
                .unwrap();
            main_pass.add_pipeline(
                pipelines.add_default(build_forward_pipeline(&mut shaders)),
                vec![Box::new(AssignedMeshesDrawTarget)],
            );
        }

        // TODO: replace these with "autowire" groups
        self.add_node_edge(node::STANDARD_MATERIAL, base_render_graph::node::MAIN_PASS)
            .unwrap();
        self.add_node_edge(node::LOCAL_TO_WORLD, base_render_graph::node::MAIN_PASS)
            .unwrap();
        self.add_node_edge(node::LIGHTS, base_render_graph::node::MAIN_PASS)
            .unwrap();
        self
    }
}
