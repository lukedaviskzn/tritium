use std::{collections::HashMap, any::TypeId};

use crate::{util::{Uid, AsAny}, renderer::Renderable};

mod script;

pub use script::*;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct NodeId(Uid);

pub trait Component: AsAny {
    fn as_renderable(&self) -> Option<&dyn Renderable> { None }
}

pub struct NodeDescriptor {
    node_id: NodeId,
    pub name: String,
    components: HashMap<TypeId, Box<dyn Component>>,
    pub children: Vec<Node>,
}

impl NodeDescriptor {
    pub fn id(&self) -> NodeId {
        self.node_id
    }

    /// Replaces existing component if present
    pub fn add_component<T: Component + 'static>(&mut self, component: T) {
        self.components.insert(TypeId::of::<T>(), Box::new(component));
    }

    pub fn has_component<T: Component + 'static>(&self) -> bool {
        self.components.contains_key(&TypeId::of::<T>())
    }

    pub fn get_component<T: Component + 'static>(&self) -> Option<&T> {
        let component = self.components.get(&TypeId::of::<T>())?.as_any();
        component.downcast_ref::<T>()
    }

    pub fn get_component_mut<T: Component + 'static>(&mut self) -> Option<&mut T> {
        let component = self.components.get_mut(&TypeId::of::<T>())?.as_any_mut();
        component.downcast_mut::<T>()
    }

    pub fn get_components(&self) -> Vec<&Box<dyn Component>> {
        let components: Vec<_> = self.components.values().collect();
        components
    }

    pub fn get_components_mut(&mut self) -> Vec<&mut Box<dyn Component>> {
        let components: Vec<_> = self.components.values_mut().collect();
        components
    }
}

pub struct Node {
    pub desc: NodeDescriptor,
    pub scripts: Vec<Box<dyn NodeScript>>,
}

impl Node {
    pub fn builder(name: &str) -> NodeBuilder {
        NodeBuilder::new(name)
    }

    fn new(name: &str, children: Vec<Node>) -> Node {
        Node {
            desc: NodeDescriptor {
                node_id: NodeId(Uid::new()),
                name: name.into(),
                components: hashmap!{},
                children,
            },
            scripts: vec![],
        }
    }

    pub fn id(&self) -> NodeId {
        self.desc.node_id
    }

    pub fn add_script<S: NodeScript + 'static>(&mut self, script: S) {
        self.scripts.push(Box::new(script));
    }

    /// Replaces existing component if present
    pub fn add_component<T: Component + 'static>(&mut self, component: T) {
        self.desc.add_component(component)
    }

    pub fn has_component<T: Component + 'static>(&self) -> bool {
        self.desc.has_component::<T>()
    }

    pub fn get_component<T: Component + 'static>(&self) -> Option<&T> {
        self.desc.get_component()
    }

    pub fn get_component_mut<T: Component + 'static>(&mut self) -> Option<&mut T> {
        self.desc.get_component_mut()
    }

    pub fn get_components(&self) -> Vec<&Box<dyn Component>> {
        self.desc.get_components()
    }

    pub fn get_components_mut(&mut self) -> Vec<&mut Box<dyn Component>> {
        self.desc.get_components_mut()
    }

    pub fn traverse<F: FnMut(&Node) -> ()>(&self, mut visit: F) {
        visit(self);

        for child in &self.desc.children {
            child.traverse(&mut visit);
        }
    }

    pub fn traverse_mut<F: FnMut(&mut Node) -> ()>(&mut self, visit: &mut F) {
        visit(self);

        for child in &mut self.desc.children {
            child.traverse_mut(visit);
        }
    }

    pub fn traverse_if<F: FnMut(&Node) -> bool, G: FnMut(&Node) -> ()>(&self, mut predicate: F, mut visit: G) {
        if predicate(self) {
            visit(self);
        }

        for child in &self.desc.children {
            child.traverse_if(&mut predicate, &mut visit);
        }
    }

    pub fn traverse_if_mut<F: FnMut(&Node) -> bool, G: FnMut(&mut Node) -> ()>(&mut self, predicate: &mut F, visit: &mut G) {
        if predicate(self) {
            visit(self);
        }

        for child in &mut self.desc.children {
            child.traverse_if_mut(predicate, visit);
        }
    }

    pub fn find<F: Fn(&Node) -> bool>(&self, predicate: F) -> Option<&Node> {
        let mut stack = vec![self];

        loop {
            if let Some(node) = stack.pop() {
                if predicate(node) {
                    return Some(node);
                }
                for child in &node.desc.children {
                    stack.push(child);
                }
            } else {
                return None;
            }
        }
    }

    pub fn find_mut<F: FnMut(&Node) -> bool>(&mut self, mut predicate: F) -> Option<&mut Node> {
        let mut stack = vec![self];

        loop {
            if let Some(node) = stack.pop() {
                if predicate(node) {
                    return Some(node);
                }
                for child in &mut node.desc.children {
                    stack.push(child);
                }
            } else {
                return None;
            }
        }
    }

    pub fn find_by_id(&self, id: &NodeId) -> Option<&Node> {
        self.find(|node| node.desc.id() == *id)
    }

    pub fn find_by_id_mut(&mut self, id: &NodeId) -> Option<&mut Node> {
        self.find_mut(|node| node.desc.id() == *id)
    }

    pub fn find_by_name(&self, name: &str) -> Option<&Node> {
        self.find(|node| node.desc.name == name)
    }

    pub fn find_by_name_mut(&mut self, name: &str) -> Option<&mut Node> {
        self.find_mut(|node| node.desc.name == name)
    }
    
    pub fn find_all<F: Fn(&Node) -> bool>(&self, predicate: F) -> Vec<&Node> {
        let mut stack = vec![self];

        let mut result = vec![];

        loop {
            if let Some(node) = stack.pop() {
                if predicate(node) {
                    result.push(node);
                }
                for child in &node.desc.children {
                    stack.push(child);
                }
            } else {
                return result;
            }
        }
    }
    
    // pub fn find_all_mut<F: Fn(&Node) -> bool>(&mut self, predicate: F) -> Vec<&mut Node> {
    //     let mut stack = vec![self];
    //     let mut result = vec![];
    //     loop {
    //         if let Some(node) = stack.pop() {
    //             if predicate(node) {
    //                 result.push(node);
    //             }
    //             for child in &mut node.desc.children {
    //                 stack.push(child);
    //             }
    //         } else {
    //             return result;
    //         }
    //     }
    // }
}

pub struct NodeBuilder {
    node: Node,
}

impl NodeBuilder {
    fn new(name: &str) -> NodeBuilder {
        NodeBuilder {
            node: Node::new(name, vec![]),
        }
    }

    pub fn from_node(node: Node) -> NodeBuilder {
        NodeBuilder {
            node,
        }
    }

    // pub fn with_script<S: NodeScript + 'static>(name: &str, script: S) -> NodeBuilder {
    //     NodeBuilder {
    //         node: Node::new(name, script, vec![]),
    //     }
    // }

    pub fn add_component<T: Component + 'static>(mut self, component: T) -> NodeBuilder {
        self.node.add_component(component);
        self
    }

    pub fn add_script<S: NodeScript + 'static>(mut self, script: S) -> NodeBuilder {
        self.node.add_script(script);
        self
    }

    pub fn add_child(mut self, node: Node) -> NodeBuilder {
        self.node.desc.children.push(node);
        self
    }

    pub fn build(self) -> Node {
        self.node
    }
}
