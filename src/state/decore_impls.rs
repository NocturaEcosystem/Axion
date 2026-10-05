use std::{collections::HashMap, process::{Command, Stdio}};

use smithay::{reexports::wayland_server::{Resource, protocol::wl_surface::WlSurface}, utils::{Logical, Physical, Point, Size}};
use tracing::warn;

use crate::state::{DecorationManager, decoration};
use slint::ComponentHandle; 

use std::io::Write;
impl DecorationManager {
    pub fn new() -> Self {
        let mut decores = HashMap::<smithay::reexports::wayland_server::backend::ObjectId, decoration>::new();
        let mut pids = vec![];
        Self {
            decores,
            pids
        }
    }

    pub fn add_surface_for_window(&mut self, id: smithay::reexports::wayland_server::backend::ObjectId) {
        let decore_pid = decoration::new();
        self.pids.push(decore_pid.1);
        self.decores.insert(id, decore_pid.0);
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
    pub fn new() -> (Self, u32) {
        let mut child = Command::new("cargo")
            .arg("run")
            .arg("--bin")
            .arg("axion-decorator")
            .stdin(Stdio::piped())
            .stderr(Stdio::inherit())
            .stdout(Stdio::inherit())
            .spawn()
            .expect("Failed to launch UI process");
        
        {
            if let Some(ref mut stdin) = child.stdin {
                writeln!(stdin, "{}", "SET_POS:10:10").unwrap();
            }
        }

        let id = child.id();

        let instance = Self {
            width: 800,
            height: 20,
            x: 10,
            y: 10,
            title: "No title".into(),
            instance: true,
            child
        };

        (instance, id)
    }

    pub fn send_request(&mut self, request: String) {
        if let Some(ref mut stdin) = self.child.stdin {
            writeln!(stdin, "{}", request).unwrap();
        } else {
            println!("FAIL")
        }
    }
    pub fn changePos(&mut self, point: Point<u32, Physical>) {
        self.x = point.x;
        self.y = point.y;
        self.refreshState("pos");
    }
    pub fn changeSize(&mut self, size: Size<u32, Physical>) {
        self.width = size.w;
        self.refreshState("width");

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

    pub fn refreshState(&mut self, target: &str) {
        match target {
            "title" => {
                let req = format!("SET_TITLE:{}", self.title);
                self.send_request(req);
            },
            "pos" => {
                let req = format!("SET_POS:{}:{}", self.x, self.y);
                self.send_request(req);
            }
            "width" => {
                let req = format!("SET_WIDTH:{}", self.width);
                self.send_request(req);
            }
            _ => {}
        }
    }
}