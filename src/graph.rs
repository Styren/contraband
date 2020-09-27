use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug)]
pub struct Value<T>(pub T);

impl<T> std::ops::Deref for Value<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T: Clone> Clone for Value<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

pub trait Injected: Send + Sync {
    type Output: Injected;
    fn resolve(graph: &mut Graph, imported_graphs: &[&Graph]) -> Self::Output
    where
        Self: Sized;
}

impl<T: Send + Sync> Injected for Value<T> {
    type Output = Self;
    fn resolve(_graph: &mut Graph, _imported_graphs: &[&Graph]) -> Self::Output {
        panic!(
            "Data type has not been provided: {}",
            std::any::type_name::<T>()
        )
    }
}

impl<T: Injected<Output = T>> Injected for Arc<T> {
    type Output = Self;
    fn resolve(graph: &mut Graph, imported_graphs: &[&Graph]) -> Self::Output {
        T::resolve(graph, imported_graphs).into()
    }
}

#[derive(Clone, Debug, Default)]
pub struct Graph {
    map: HashMap<TypeId, Arc<(dyn Send + Sync + Any)>>,
}

impl Graph {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn filter_by(&self, set: std::collections::HashSet<TypeId>) -> Self {
        let mut new = self.clone();
        new.map.retain(|&k, _| set.contains(&k));
        new
    }

    pub fn search_all<'a, T: 'static>(graphs: &'a [&Self]) -> Option<&'a T> {
        for graph in graphs {
            if let Some(ret) = graph.get_node::<T>() {
                return Some(ret);
            }
        }
        None
    }

    pub fn get_node<T: 'static>(&self) -> Option<&T> {
        self.map
            .get(&TypeId::of::<T>())
            .and_then(|boxed| (&**boxed as &(dyn Any + Send + 'static)).downcast_ref())
    }

    pub fn get_ptr<T: 'static>(&self) -> Option<Arc<T>> {
        self.map.get(&TypeId::of::<T>()).and_then(|boxed| {
            (&**boxed as &(dyn Any + Send + 'static))
                .downcast_ref::<Arc<T>>()
                .cloned()
        })
    }

    pub fn contains<T: 'static>(&self) -> bool {
        self.map.get(&TypeId::of::<T>()).is_some()
    }

    pub fn provide<T: Send + Sync + 'static>(&mut self, t: Arc<T>) -> &T {
        let exists = self.contains::<T>();
        if !exists {
            self.map.insert(TypeId::of::<T>(), t);
        }
        self.get_node::<T>().unwrap()
    }

    pub fn resolve<'a, T: Injected + Sync + Send + 'static>(
        &'a mut self,
        imports: &'a [&Self],
    ) -> &'a T {
        let exists = self.contains::<T>();
        for graph in imports {
            let exists = graph.contains::<T>();
            if exists {
                return graph.get_node::<T>().unwrap();
            }
        }
        if !exists {
            let new = T::resolve(self, imports);
            self.map.insert(TypeId::of::<T>(), Arc::new(new));
        }
        self.get_node::<T>().unwrap()
    }
}
