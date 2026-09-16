use std::collections::BTreeMap;
use std::ops::Bound;
use std::time::Duration;
use std::{fs::File, sync::Mutex};

use std::error;
use smithay::backend::allocator;
use smithay::backend::renderer::element::surface::render_elements_from_surface_tree;
use smithay::backend::renderer::element::{AsRenderElements, Kind};
use smithay::backend::renderer::gles::GlesTexture;
use smithay::backend::renderer::{ImportAll, ImportMem};
use smithay::backend::renderer::element::texture::{TextureBuffer, TextureRenderElement};
use smithay::render_elements;
use smithay::{backend::renderer::{self, Renderer, Texture, element::surface::WaylandSurfaceRenderElement, gles::GlesRenderer}, input::pointer::{CursorImageAttributes, CursorImageStatus}, utils::Point, wayland::compositor};
use xcursor::CursorTheme;
use xcursor::parser::parse_xcursor;
use crate::state::{NocturaCursor};
use std::io::Read;

fn getCursorData() -> Result<Vec<u8>, Box<dyn error::Error>> {
    let theme = std::env::var("XCURSOR_THEME").ok().unwrap_or(String::from("default"));

    let ct = CursorTheme::load(&theme);
    let cp = ct.load_icon("left_ptr").unwrap();
    let mut cf = File::open(&cp).unwrap();
    let mut res: Vec<u8> = vec![];
    cf.read_to_end(&mut res).unwrap();
    Ok(res)
}

impl<T: Texture> NocturaCursor<T> {
    pub fn try_new<R: Renderer>(renderer: &mut R) -> Option<Self>
        where R: Renderer<TextureId = T> + ImportMem {
        let cursor_data = getCursorData().ok();
        let size = std::env::var("XCURSOR_SIZE").ok().and_then(|s| {
            s.parse::<i32>().ok()
        }).unwrap_or(24);
        if cursor_data.is_none() {
            return None
        }
        let cursor_data = cursor_data.unwrap(); // unwrap is safe here
        let ci = parse_xcursor(&cursor_data).unwrap().into_iter().filter(
            move |image | image.width == size as u32 && image.height == size as u32
        );
        let mut map_of_image = BTreeMap::new();
        let mut td: u64 = 0;
        ci.for_each(|image| {
            td += image.delay as u64;
            let texture = 
            renderer.import_memory(&image.pixels_rgba.as_slice(), allocator::Fourcc::Abgr8888, (size, size).into(), false).unwrap();
            let buff = TextureBuffer::from_texture(renderer, texture, 1, smithay::utils::Transform::Normal, None);
            map_of_image.insert(td, buff); // becomes aranged
        });
        Some(Self {
            cs: CursorImageStatus::default_named(),
            whole_stamp: td,
            current_stamp: 0,
            map_of_image,
        })
    }
    pub fn current_delay(&mut self, current_duration: Duration) {
        self.current_stamp = current_duration.as_millis() as u64 % self.whole_stamp;
    }
    pub fn current_texture(&self) -> Option<&TextureBuffer<T>> {
        self.map_of_image.range(     // get the range, then get the smallest number where
//                                      self.current_stamp larger. for example, if we have:
//                                      50 ms | textureBuffer
//                                      100 ms | textureBuffer
//                                      150 ms | textureBuffer
//                                      and current delay is 110ms, selected would be:
//                                      ``100 ms | textureBuffer
//                                      150 ms | textureBuffer``
           ( Bound::Excluded(self.current_stamp),
            Bound::Unbounded)
        ).next()                    // from that:
//                                      100 ms | textureBuffer
//                                      150 ms | textureBuffer
//                                      .next() is going to select the first one, ``100 ms | textureBuffer``
        .map(|(_, texture_buffer)| texture_buffer) // this cmd extract ``textureBuffer`` from  ``100 ms | textureBuffer``
    }
}

render_elements! {
    pub PointerRenderElement<R> where
        R: ImportAll;

    Surface = WaylandSurfaceRenderElement<R>,
    Texture = TextureRenderElement<R::TextureId>,
}

impl<T: Texture + Clone + Send + 'static, R> AsRenderElements<R> for NocturaCursor<T>
where
    R: Renderer<TextureId = T> + ImportAll + ImportMem,
{
    type RenderElement = PointerRenderElement<R>;
    fn render_elements<C: From<Self::RenderElement>>(
        &self,
        renderer: &mut R,
        location: Point<i32, smithay::utils::Physical>,
        scale: smithay::utils::Scale<f64>,
        alpha: f32,
    )  -> Vec<C> where 
    C: From<PointerRenderElement<R>>
    {
        match &self.cs {
            CursorImageStatus::Hidden => vec![],
            CursorImageStatus::Named(_) => {
                let texture = self.current_texture();
                if texture.is_none() {
                    return vec![]
                }
                let texture = texture.unwrap(); // safe to unwrap here
                vec![PointerRenderElement::<R>::from(TextureRenderElement::from_texture_buffer(location.to_f64(), texture, None, None, None, Kind::Cursor)).into()]
            }
            CursorImageStatus::Surface(surf) => {
                render_elements_from_surface_tree(renderer, &surf, location, scale, alpha, Kind::Cursor).into_iter().map(C::from).collect()
            }
        }
    }
}