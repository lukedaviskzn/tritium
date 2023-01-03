use std::{sync::Arc, marker::PhantomData, hash::Hash, any::Any};

use crate::{util::{Uid, AsAny}, node::{Component, NodeDescriptor}, renderer::{Renderable, RenderInput, Renderer, RenderableResource, Shader}};

use super::Resources;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct HandleId(Uid);

// todo: separate into separate Handle and WeakHandle structs

pub trait HandlesResource<T> {
    fn id(&self) -> HandleId;
    fn valid(&self) -> bool;
    fn downgrade(&self) -> WeakHandle<T>;
}

pub struct Handle<T>(HandleId, Arc<PhantomData<T>>);

pub struct WeakHandle<T>(HandleId, std::sync::Weak<PhantomData<T>>);

impl<T: 'static> Handle<T> {
    pub(super) fn new() -> Handle<T> {
        Handle(HandleId(Uid::new()), Arc::new(PhantomData))
    }

    pub(super) fn id(&self) -> HandleId {
        self.0
    }

    pub fn downgrade(&self) -> WeakHandle<T> {
        WeakHandle(self.0, Arc::downgrade(&self.1))
    }

    pub fn get<'s, 'a>(&'s self, resources: &'a Resources) -> &'a T where 'a: 's {
        resources.get(self).expect("Attempted to fetch resources with invalid strong handle, this is a bug.")
    }

    pub fn get_mut<'s, 'a>(&'s mut self, resources: &'a mut Resources) -> &'a mut T where 'a: 's {
        resources.get_mut(self).expect("Attempted to fetch resources with invalid strong handle, this is a bug.")
    }

    pub fn set(&self, resources: &mut Resources, resource: T) {
        resources.set(self, resource);
    }
}

impl<T: 'static> WeakHandle<T> {
    pub fn upgrade(&self) -> Option<Handle<T>> {
        let arc = std::sync::Weak::upgrade(&self.1)?;
        Some(Handle(self.0, arc))
    }

    pub(super) fn id(&self) -> HandleId {
        self.0
    }

    pub fn get<'s, 'a>(&'s self, resources: &'a Resources) -> Option<&'a T> where 'a: 's {
        resources.get(self)
    }

    pub fn get_mut<'s, 'a>(&'s mut self, resources: &'a mut Resources) -> Option<&'a mut T> where 'a: 's {
        resources.get_mut(self)
    }

    pub fn set(&self, resources: &mut Resources, resource: T) {
        resources.set(self, resource);
    }
}

impl<T: 'static> HandlesResource<T> for Handle<T> {
    fn id(&self) -> HandleId {
        self.0
    }

    fn valid(&self) -> bool {
        true
    }

    fn downgrade(&self) -> WeakHandle<T> {
        self.downgrade()
    }
}

impl<T: 'static> HandlesResource<T> for WeakHandle<T> {
    fn id(&self) -> HandleId {
        self.0
    }

    fn valid(&self) -> bool {
        std::sync::Weak::strong_count(&self.1) > 0
    }

    fn downgrade(&self) -> WeakHandle<T> {
        self.clone()
    }
}

// Component/Renderable for Handle
impl<T: 'static> AsAny for Handle<T> {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

impl<T: RenderableResource + 'static> Component for Handle<T> {
    fn as_renderable(&self) -> Option<&dyn Renderable> { Some(self) }
}

impl<T: RenderableResource + 'static> Renderable for Handle<T> {
    fn render_inputs(&self, node: &NodeDescriptor, renderer: &Renderer, resources: &mut Resources) -> Vec<RenderInput> {
        match resources.get(self) {
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

// Component/Renderable for WeakHandle
impl<T: 'static> AsAny for WeakHandle<T> {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

impl<T: RenderableResource + 'static> Component for WeakHandle<T> {
    fn as_renderable(&self) -> Option<&dyn Renderable> { Some(self) }
}

impl<T: RenderableResource + 'static> Renderable for WeakHandle<T> {
    fn render_inputs(&self, node: &NodeDescriptor, renderer: &Renderer, resources: &mut Resources) -> Vec<RenderInput> {
        match resources.get(self) {
            Some(resource) => RenderableResource::render_inputs(resource, node, renderer, resources),
            None => vec![],
        }
    }
}

impl Component for WeakHandle<Shader> {
    fn as_renderable(&self) -> Option<&dyn Renderable> { Some(self) }
}

impl Renderable for WeakHandle<Shader> {
    fn render_inputs(&self, _node: &NodeDescriptor, _renderer: &Renderer, _resources: &mut Resources) -> Vec<RenderInput> {
        // vec![RenderInput::new("shader", RenderInputStorage::Shader(self.clone()))]
        match self.upgrade() {
            Some(handle) => vec![RenderInput::Shader(handle)],
            None => vec![],
        }
    }
}

// Manual implmentation of Debug, Clone, Hash, PartialEq, Eq, since generic T interferes with derive

impl<T> std::fmt::Debug for Handle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple(&format!("Handle<{}>", std::any::type_name::<T>())).field(&self.0).finish()
    }
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Handle<T> {
        Handle(self.0, Arc::clone(&self.1))
    }
}

impl<T: 'static> Hash for Handle<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id().hash(state);
    }
}

impl<T: 'static> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl<T: 'static> Eq for Handle<T> {}

// Manual implmentation for weak handle

impl<T> std::fmt::Debug for WeakHandle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple(&format!("WeakHandle<{}>", std::any::type_name::<T>())).field(&self.0).finish()
    }
}

impl<T> Clone for WeakHandle<T> {
    fn clone(&self) -> WeakHandle<T> {
        WeakHandle(self.0, std::sync::Weak::clone(&self.1))
    }
}

impl<T: 'static> Hash for WeakHandle<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id().hash(state);
    }
}

impl<T: 'static> PartialEq for WeakHandle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl<T: 'static> Eq for WeakHandle<T> {}
