use std::{
    any::{
        type_name,
        Any,
        TypeId,
    },
    collections::HashMap,
};

#[derive(Default)]
pub struct Resources {
    resources: HashMap<TypeId, Box<dyn Any>>,
}

impl Resources {
    pub fn insert<R: 'static>(&mut self, resource: R) {
        self.resources.insert(TypeId::of::<R>(), Box::new(resource));
    }

    pub fn get<R: 'static>(&self) -> Option<&R> {
        self.resources
            .get(&TypeId::of::<R>())
            .map(|resource| resource.downcast_ref().unwrap())
    }

    pub fn try_get<R: 'static>(&self) -> Result<&R, ResourceNotFound> {
        self.get().ok_or_else(|| {
            ResourceNotFound {
                resource: type_name::<R>(),
            }
        })
    }

    pub fn get_mut_or_insert_default<R: Default + 'static>(&mut self) -> &mut R {
        self.resources
            .entry(TypeId::of::<R>())
            .or_insert_with(|| Box::new(R::default()))
            .downcast_mut()
            .unwrap()
    }

    pub fn get_mut<R: 'static>(&mut self) -> Option<&mut R> {
        self.resources
            .get_mut(&TypeId::of::<R>())
            .map(|resource| resource.downcast_mut().unwrap())
    }

    pub fn try_get_mut<R: 'static>(&mut self) -> Result<&mut R, ResourceNotFound> {
        self.get_mut().ok_or_else(|| {
            ResourceNotFound {
                resource: type_name::<R>(),
            }
        })
    }
}

#[derive(Debug, thiserror::Error)]
#[error("resource not found: {resource}")]
pub struct ResourceNotFound {
    pub resource: &'static str,
}
