use std::{
    collections::HashMap,
    fmt::{
        Debug,
        Display,
    },
};

use guillotiere::{
    AllocId,
    AtlasAllocator,
    Rectangle,
    Size,
};
use image::{
    GenericImage,
    RgbaImage,
};
use kardashev_protocol::assets::TextureCrop;

use crate::Error;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum AtlasBuilderId {
    Default,
    Named(String),
}

impl Display for AtlasBuilderId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Default => write!(f, "default"),
            Self::Named(name) => write!(f, "{name}"),
        }
    }
}

pub struct AtlasBuilder<D> {
    allocator: AtlasAllocator,
    allocations: HashMap<AllocId, Allocation<D>>,
}

impl<D: Debug> Debug for AtlasBuilder<D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AtlasBuilder")
            .field("allocations", &self.allocations)
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
struct Allocation<D> {
    rectangle: Rectangle,
    image: RgbaImage,
    data: D,
}

impl<D> Default for AtlasBuilder<D> {
    fn default() -> Self {
        Self {
            allocator: AtlasAllocator::new(Self::INITIAL_SIZE),
            allocations: HashMap::new(),
        }
    }
}

impl<D> AtlasBuilder<D> {
    const INITIAL_SIZE: Size = Size::new(1024, 1024);

    pub fn insert(&mut self, image: RgbaImage, data: D) -> Result<(), Error> {
        let image_size = image.dimensions();

        let allocation = loop {
            if let Some(allocation) = self
                .allocator
                .allocate(Size::new(image_size.0 as i32, image_size.1 as i32))
            {
                break allocation;
            }
            else {
                let old_size = self.allocator.size();
                let new_size = old_size * 2;
                let changes = self.allocator.resize_and_rearrange(new_size);
                if !changes.failures.is_empty() {
                    panic!("failed to grow atlas allocator");
                }
                let changes = changes
                    .changes
                    .into_iter()
                    .map(|change| (change.old.id, change.new))
                    .collect::<HashMap<_, _>>();
                self.allocations = self
                    .allocations
                    .drain()
                    .map(|(old_id, mut allocation)| {
                        let new = changes.get(&old_id).unwrap();
                        allocation.rectangle = new.rectangle;
                        (new.id, allocation)
                    })
                    .collect();
            }
        };

        self.allocations.insert(
            allocation.id,
            Allocation {
                rectangle: allocation.rectangle,
                image,
                data,
            },
        );

        Ok(())
    }

    pub fn finish(self) -> Result<Atlas<D>, Error> {
        let image_size = self.allocator.size();
        let image_size = [image_size.width as u32, image_size.height as u32];

        let mut image = RgbaImage::new(image_size[0], image_size[1]);
        let mut allocations = Vec::with_capacity(self.allocations.len());

        for (_, allocation) in self.allocations {
            let allocation_size = allocation.rectangle.size();
            image.copy_from(
                &allocation.image,
                allocation.rectangle.min.x as u32,
                allocation.rectangle.min.y as u32,
            )?;
            allocations.push((
                allocation.data,
                TextureCrop {
                    x: allocation.rectangle.min.x as u32,
                    y: allocation.rectangle.min.y as u32,
                    w: allocation_size.width as u32,
                    h: allocation_size.height as u32,
                },
            ));
        }

        Ok(Atlas {
            allocations,
            image,
            image_size,
        })
    }
}

#[derive(Debug)]
pub struct Atlas<D> {
    pub allocations: Vec<(D, TextureCrop)>,
    pub image: RgbaImage,
    pub image_size: [u32; 2],
}
