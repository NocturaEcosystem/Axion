use std::collections::HashMap;

use smithay::{reexports::wayland_server::{Resource, protocol::wl_surface::WlSurface}, utils::{Logical, Physical, Point, Size}};

use crate::state::{DecorationManager, decoration};

impl DecorationManager {
    pub fn new() -> Self {
        let mut decores = HashMap::<smithay::reexports::wayland_server::backend::ObjectId, decoration>::new();
        Self {
            decores
        }
    }

    pub fn add_surface_for_window(&mut self, id: smithay::reexports::wayland_server::backend::ObjectId) {
        self.decores.insert(id, decoration::new());
    }

    pub fn change_title(&mut self, id: &smithay::reexports::wayland_server::backend::ObjectId, title: String) {
        if let Some(val) = self.decores.get_mut(id) {
            val.title = title;
        }
    }

    pub fn delete_decore(&mut self, id: &smithay::reexports::wayland_server::backend::ObjectId) {
        self.decores.remove(id);
    }

    pub fn getSurfaceDecore(&mut self, id: &smithay::reexports::wayland_server::backend::ObjectId) -> Option<&mut decoration> {
        let val = self.decores.get_mut(&id);
        val
    }
}

impl decoration {
    pub fn new() -> Self {
        Self {
            width: 800,
            height: 20,
            x: 10,
            y: 10,
            title: "No title".into()
        }
    }
    pub fn changePos(&mut self, point: Point<u32, Physical>) {
        self.x = point.x;
        self.y = point.y;
    }
    pub fn changeSize(&mut self, size: Size<u32, Physical>) {
        self.width = size.w;
    }
    pub fn states(&self) {
        println!(
            "TEMP DEV STATES
            x: {}, y: {}, w: {}, h: {}
            ",
            self.x,
            self.y,
            self.width,
            self.height
        )
    }
}