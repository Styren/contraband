use std::any::TypeId;
use super::graph::{Graph, Injected};
use std::sync::Arc;
use std::collections::{HashMap, HashSet};
use actix_web::web::ServiceConfig;

pub trait ServiceFactory: Send + Sync {
    fn register(&self, app: &mut ServiceConfig);
}

pub(crate) struct Context {
    pub(crate) global_providers: Graph,
    pub(crate) modules: HashMap<TypeId, Arc<ResolvedModule>>,
}

#[derive(Default)]
pub struct Module {
    exported_providers: HashSet<TypeId>,
    entities: HashSet<TypeId>,
    imports: Vec<Box<dyn FnOnce(&mut ResolvedModule, &mut Context)>>,
    provider_values: Vec<Box<dyn FnOnce(&mut ResolvedModule)>>,
    providers: Vec<Box<dyn FnOnce(&mut ResolvedModule, &mut Context)>>,
    controllers: Vec<Box<dyn FnOnce(&mut ResolvedModule, &mut Context)>>,
}

impl Module {
    pub fn new() -> Self {
        Self {
            exported_providers: HashSet::new(),
            entities: HashSet::new(),
            imports: Vec::new(),
            provider_values: Vec::new(),
            providers: Vec::new(),
            controllers: Vec::new(),
        }
    }

    pub fn import<T: ModuleFactory + 'static>(mut self) -> Self {
        self.imports.push(Box::new(|module, ctx| {
            if let Some(resolved_module) = ctx.modules.get(&TypeId::of::<T>()) {
                module.imported_modules.push(resolved_module.clone());
            } else {
                let new_module = Arc::new(T::get_module().build(ctx));
                ctx.modules.insert(TypeId::of::<T>(), new_module.clone());
                module.imported_modules.push(new_module);
            }
        }));
        self
    }

    pub fn export<T>(mut self) -> Self where T: Injected + Send + Sync + 'static {
        self.exported_providers.insert(TypeId::of::<T>());
        self
    }

    pub fn export_val<T>(mut self, _: &T) -> Self where T: Injected + Send + Sync + 'static {
        self.exported_providers.insert(TypeId::of::<T>());
        self
    }

    pub fn provide_value<T: Sync + Send + Clone>(mut self, t: T) -> Self where T: 'static {
        self.provider_values.push(Box::new(|module| {
            module.graph.provide(Arc::new(t));
        }));
        self.entities.insert(TypeId::of::<T>());
        self
    }

    pub fn provide<T>(mut self) -> Self where T: Injected<Output = T> + 'static {
        self.providers.push(Box::new(|module, ctx| {
            let mut imported_graphs = vec!(&ctx.global_providers);
            for module in &module.imported_modules {
                imported_graphs.push(&module.exported_graph);
            }
            module.graph.resolve::<Arc<T>>(&imported_graphs);
        }));
        self.entities.insert(TypeId::of::<T>());
        self
    }

    pub fn controller<T>(mut self) -> Self where T: Injected<Output = T> + ServiceFactory + 'static {
        self.controllers.push(Box::new(|module, ctx| {
            let mut imported_graphs = vec!(&ctx.global_providers);
            for module in &module.imported_modules {
                imported_graphs.push(&module.exported_graph);
            }
            let resolved = T::resolve(&mut module.graph, &imported_graphs);
            module.controllers.push(Arc::new(resolved));
        }));
        self.entities.insert(TypeId::of::<T>());
        self
    }

    pub(crate) fn build(self, ctx: &mut Context) -> ResolvedModule {
        let mut module = ResolvedModule::new();
        for import in self.imports {
            import(&mut module, ctx);
        }
        for provided_value in self.provider_values {
            provided_value(&mut module);
        }
        for provider in self.providers {
            provider(&mut module, ctx);
        }
        for controller in self.controllers {
            controller(&mut module, ctx);
        }
        module.exported_graph = module.graph.filter_by(self.exported_providers);
        module
    }
}

#[derive(Clone)]
pub(crate) struct ResolvedModule {
    pub(crate) graph: Graph,
    pub(crate) imported_modules: Vec<Arc<Self>>,
    exported_graph: Graph,
    pub(crate) controllers: Vec<Arc<dyn ServiceFactory>>,
}

impl ResolvedModule {
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
            exported_graph: Graph::new(),
            imported_modules: Vec::new(),
            controllers: Vec::new(),
        }
    }
}

pub trait ModuleFactory: Sized {
    fn get_module() -> Module;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::module::Module;
    use crate::graph::Value;

    fn get_empty_ctx() -> Context {
        Context {
            global_providers: Graph::new(),
            modules: HashMap::new(),
        }
    }

    #[test]
    fn test_provide_value_get_resolved() {
        let value = Value("test_str");
        let mut ctx = get_empty_ctx();
        let resolved = Module::new()
            .provide_value(value)
            .build(&mut ctx);
        assert_eq!(**resolved.graph.get_node::<Value<&str>>().unwrap(), "test_str");
    }

    #[test]
    fn test_imported_value_is_reachable() {
        struct ExportingModule;
        impl ModuleFactory for ExportingModule {
            fn get_module() -> Module {
                let value = Value("test_str");
                Module::new()
                    .export_val(&value)
                    .provide_value(value)
            }
        }

        let mut ctx = get_empty_ctx();
        let resolved = Module::new()
            .import::<ExportingModule>()
            .build(&mut ctx);
        assert_eq!(resolved.imported_modules.len(), 1);
        assert_eq!(
            **resolved.imported_modules[0].graph.get_node::<Value<&str>>().unwrap(),
            "test_str"
        );
    }
}
