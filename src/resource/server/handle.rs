use std::{sync::Arc, marker::PhantomData, hash::Hash, any::Any};

use crate::{util::{Uid, AsAny}, node::{Component, NodeDescriptor}, renderer::{Renderable, RenderInput, Renderer, RenderableResource, Shader}};

use super::Resources;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct HandleId(Uid);

// todo: separate into separate Handle and WeakHandle structs
pub enum Handle<T> {
    Strong(HandleId, Arc<PhantomData<T>>),
    Weak(HandleId, std::sync::Weak<PhantomData<T>>),
}

impl<T> Handle<T> {
    pub(crate) fn new_invalid() -> Handle<T> {
        Handle::new_strong().downgrade()
    }

    pub fn new_strong() -> Handle<T> {
        Handle::Strong(HandleId(Uid::new()), Arc::new(PhantomData))
    }

    /// If strong: create weak copy, if weak: create copy
    pub fn downgrade(&self) -> Handle<T> {
        match self {
            Handle::Strong(uid, marker) => Handle::Weak(*uid, Arc::downgrade(marker)),
            Handle::Weak(_, _) => self.clone(),
        }
    }

    /// If weak: create strong reference (if possible), if strong: create copy
    pub fn upgrade(&self) -> Option<Handle<T>> {
        match self {
            Handle::Strong(_, _) => Some(self.clone()),
            Handle::Weak(uid, marker) => Some(Handle::Strong(*uid, std::sync::Weak::upgrade(marker)?)),
        }
    }

    /// Holds valid reference to resource
    pub fn valid(&self) -> bool {
        match self {
            // strong handle should always be true
            Handle::Strong(_, _) => true,
            // weak handle may no longer exist
            Handle::Weak(_, marker) => std::sync::Weak::upgrade(marker).is_some(),
        }
    }

    pub fn is_strong(&self) -> bool {
        match self {
            Handle::Strong(_, _) => true,
            Handle::Weak(_, _) => false,
        }
    }

    pub fn is_weak(&self) -> bool {
        !self.is_strong()
    }

    pub(super) fn id(&self) -> HandleId {
        match self {
            Handle::Strong(uid, _) => *uid,
            Handle::Weak(uid, _) => *uid,
        }
    }
}

impl<T: 'static> AsAny for Handle<T> {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

// impl<T: !RenderableResource + 'static> Component for Handle<T> {}

// impl<T: 'static> Component for Handle<T> {}

impl<T: RenderableResource + 'static> Component for Handle<T> {
    fn as_renderable(&self) -> Option<&dyn Renderable> { Some(self) }
}

impl<T: RenderableResource + 'static> Renderable for Handle<T> {
    fn render_inputs(&self, node: &NodeDescriptor, renderer: &Renderer, resources: &mut Resources) -> Vec<RenderInput> {
        match resources.get(&self) {
            Some(resource) => RenderableResource::render_inputs(resource, node, renderer, resources),
            None => vec![],
        }
    }
}

impl Component for Handle<Shader> {
    fn as_renderable(&self) -> Option<&dyn Renderable> { Some(self) }
}

impl Renderable for Handle<Shader> {
    fn render_inputs(&self, _node: &NodeDescriptor, _renderer: &Renderer, _resources: &mut Resources) -> Vec<RenderInput> {
        // vec![RenderInput::new("shader", RenderInputStorage::Shader(self.clone()))]
        vec![RenderInput::Shader(self.clone())]
    }
}

// Manual implmentation of Debug, Clone, Hash, PartialEq, Eq, since generic T interferes with derive

impl<T> std::fmt::Debug for Handle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let type_name = std::any::type_name::<T>();
        match self {
            Self::Strong(uid, _) => f.debug_tuple(&format!("Handle<{}>::Strong", type_name)).field(uid).finish(),
            Self::Weak(uid, _) => f.debug_tuple(&format!("Handle<{}>::Weak", type_name)).field(uid).finish(),
        }
    }
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        match self {
            Self::Strong(arg0, arg1) => Self::Strong(arg0.clone(), arg1.clone()),
            Self::Weak(arg0, arg1) => Self::Weak(arg0.clone(), arg1.clone()),
        }
    }
}

impl<T> Hash for Handle<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id().hash(state);
    }
}

impl<T> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl<T> Eq for Handle<T> {}
