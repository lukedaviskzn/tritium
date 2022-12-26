mod colour;

pub use colour::*;

use crate::{world::NodeDescriptor, resource::{Resources, Handle}, input::{KeyboardManager, MouseManager}};

#[allow(unused_variables)]
pub trait EngineScript {
    // fn first_update(&mut self, node: &mut NodeDescriptor, context: &UpdateContext, resources: &mut Resources) {}
    fn pre_update(&mut self, node: &mut NodeDescriptor, context: &UpdateContext, resources: &mut Resources) {}
    fn update(&mut self, node: &mut NodeDescriptor, context: &UpdateContext, resources: &mut Resources) {}
    fn post_update(&mut self, node: &mut NodeDescriptor, context: &UpdateContext, resources: &mut Resources) {}
    fn pre_tick(&mut self, node: &mut NodeDescriptor, context: &UpdateContext, resources: &mut Resources) {}
    fn tick(&mut self, node: &mut NodeDescriptor, context: &UpdateContext, resources: &mut Resources) {}
    fn post_tick(&mut self, node: &mut NodeDescriptor, context: &UpdateContext, resources: &mut Resources) {}
    // fn extract(&mut self, node: &mut NodeDescriptor, resources: &mut Resources, renderer: &Renderer) -> RenderNode {
    //     RenderNode::new(node)
    // }
}

pub struct EmptyScript;

impl EngineScript for EmptyScript {}

pub struct ClosureScript {
    // first_update: Option<Box<dyn FnMut(&mut NodeDescriptor, &UpdateContext, &mut Resources)>>,
    pre_update: Option<Box<dyn FnMut(&mut NodeDescriptor, &UpdateContext, &mut Resources)>>,
    update: Option<Box<dyn FnMut(&mut NodeDescriptor, &UpdateContext, &mut Resources)>>,
    post_update: Option<Box<dyn FnMut(&mut NodeDescriptor, &UpdateContext, &mut Resources)>>,
    pre_tick: Option<Box<dyn FnMut(&mut NodeDescriptor, &UpdateContext, &mut Resources)>>,
    tick: Option<Box<dyn FnMut(&mut NodeDescriptor, &UpdateContext, &mut Resources)>>,
    post_tick: Option<Box<dyn FnMut(&mut NodeDescriptor, &UpdateContext, &mut Resources)>>,
    // extract: Option<Box<dyn FnMut(&mut NodeDescriptor, &mut Resources, &Renderer) -> RenderNode>>,
}

impl ClosureScript {
    fn new(
        // first_update: Option<Box<dyn FnMut(&mut NodeDescriptor, &UpdateContext, &mut Resources)>>,
        pre_update: Option<Box<dyn FnMut(&mut NodeDescriptor, &UpdateContext, &mut Resources)>>,
        update: Option<Box<dyn FnMut(&mut NodeDescriptor, &UpdateContext, &mut Resources)>>,
        post_update: Option<Box<dyn FnMut(&mut NodeDescriptor, &UpdateContext, &mut Resources)>>,
        pre_tick: Option<Box<dyn FnMut(&mut NodeDescriptor, &UpdateContext, &mut Resources)>>,
        tick: Option<Box<dyn FnMut(&mut NodeDescriptor, &UpdateContext, &mut Resources)>>,
        post_tick: Option<Box<dyn FnMut(&mut NodeDescriptor, &UpdateContext, &mut Resources)>>,
        // extract: Option<Box<dyn FnMut(&mut NodeDescriptor, &mut Resources, &Renderer) -> RenderNode>>,
    ) -> ClosureScript {
        ClosureScript {
            // first_update,
            pre_update,
            update,
            post_update,
            pre_tick,
            tick,
            post_tick,
            // extract,
        }
    }

    pub fn builder() -> ScriptBuilder {
        ScriptBuilder::new()
    }
}

impl EngineScript for ClosureScript {
    // fn first_update(&mut self, node: &mut NodeDescriptor, context: &UpdateContext, resources: &mut Resources) {
    //     if let Some(first_update) = &mut self.first_update {
    //         (first_update)(node, context, resources)
    //     }
    // }

    fn pre_update(&mut self, node: &mut NodeDescriptor, context: &UpdateContext, resources: &mut Resources) {
        if let Some(pre_update) = &mut self.pre_update {
            (pre_update)(node, context, resources)
        }
    }

    fn update(&mut self, node: &mut NodeDescriptor, context: &UpdateContext, resources: &mut Resources) {
        if let Some(update) = &mut self.update {
            (update)(node, context, resources)
        }
    }

    fn post_update(&mut self, node: &mut NodeDescriptor, context: &UpdateContext, resources: &mut Resources) {
        if let Some(post_update) = &mut self.post_update {
            (post_update)(node, context, resources)
        }
    }

    fn pre_tick(&mut self, node: &mut NodeDescriptor, context: &UpdateContext, resources: &mut Resources) {
        if let Some(pre_tick) = &mut self.pre_tick {
            (pre_tick)(node, context, resources)
        }
    }

    fn tick(&mut self, node: &mut NodeDescriptor, context: &UpdateContext, resources: &mut Resources) {
        if let Some(tick) = &mut self.tick {
            (tick)(node, context, resources)
        }
    }

    fn post_tick(&mut self, node: &mut NodeDescriptor, context: &UpdateContext, resources: &mut Resources) {
        if let Some(post_tick) = &mut self.post_tick {
            (post_tick)(node, context, resources)
        }
    }

    // fn extract(&mut self, node: &mut NodeDescriptor, resources: &mut Resources, renderer: &Renderer) -> RenderNode {
    //     if let Some(extract) = &mut self.extract {
    //         (extract)(node, resources, renderer)
    //     } else {
    //         RenderNode::new(node)
    //     }
    // }
}

pub struct ScriptBuilder {
    script: ClosureScript,
}

impl ScriptBuilder {
    pub fn new() -> ScriptBuilder {
        ScriptBuilder {
            script: ClosureScript::new(None, None, None, None, None, None),
        }
    }

    // pub fn first_update<T: FnMut(&mut NodeDescriptor, &UpdateContext, &mut Resources) + 'static>(mut self, first_update: T) -> ScriptBuilder {
    //     self.script.first_update = Some(Box::new(first_update));
    //     self
    // }

    pub fn pre_update<T: FnMut(&mut NodeDescriptor, &UpdateContext, &mut Resources) + 'static>(mut self, pre_update: T) -> ScriptBuilder {
        self.script.pre_update = Some(Box::new(pre_update));
        self
    }

    pub fn update<T: FnMut(&mut NodeDescriptor, &UpdateContext, &mut Resources) + 'static>(mut self, update: T) -> ScriptBuilder {
        self.script.update = Some(Box::new(update));
        self
    }

    pub fn post_update<T: FnMut(&mut NodeDescriptor, &UpdateContext, &mut Resources) + 'static>(mut self, post_update: T) -> ScriptBuilder {
        self.script.post_update = Some(Box::new(post_update));
        self
    }

    pub fn pre_tick<T: FnMut(&mut NodeDescriptor, &UpdateContext, &mut Resources) + 'static>(mut self, pre_tick: T) -> ScriptBuilder {
        self.script.pre_tick = Some(Box::new(pre_tick));
        self
    }

    pub fn tick<T: FnMut(&mut NodeDescriptor, &UpdateContext, &mut Resources) + 'static>(mut self, tick: T) -> ScriptBuilder {
        self.script.tick = Some(Box::new(tick));
        self
    }

    pub fn post_tick<T: FnMut(&mut NodeDescriptor, &UpdateContext, &mut Resources) + 'static>(mut self, post_tick: T) -> ScriptBuilder {
        self.script.post_tick = Some(Box::new(post_tick));
        self
    }

    // pub fn extract<T: FnMut(&mut NodeDescriptor, &mut Resources, &Renderer) -> RenderNode + 'static>(mut self, extract: T) -> ScriptBuilder {
    //     self.script.extract = Some(Box::new(extract));
    //     self
    // }

    pub fn build(self) -> ClosureScript {
        self.script
    }
}

pub struct FrameCounter {
    pub current_frame: usize,
    pub last_frame_time: std::time::Instant,

    pub current_tick: usize,
    pub last_tick_time: std::time::Instant,

    pub accumulator: std::time::Duration,
}

impl FrameCounter {
    pub fn new() -> FrameCounter {
        FrameCounter {
            current_frame: 0,
            last_frame_time: std::time::Instant::now(),

            current_tick: 0,
            last_tick_time: std::time::Instant::now(),

            accumulator: std::time::Duration::ZERO,
        }
    }
}

pub struct UpdateContext {
    pub window_size: glam::Vec2,
    pub window_focused: bool,
    pub delta_time: f32,
    pub keyboard: Handle<KeyboardManager>,
    pub mouse: Handle<MouseManager>,
}
