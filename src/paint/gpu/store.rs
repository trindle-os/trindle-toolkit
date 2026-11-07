//! What the drawing service asks of the painter: a store for each program, letters shared between programs, places for uploads.

use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::hash::BuildHasher;
use std::rc::Rc;

use tiny_skia::Pixmap;

use super::{Atlas, Gpu, Letters, Spot, Store, Target, SHARE_MOST};
#[cfg(test)]
use super::SIDE;
use crate::sharp::{Mask, MaskKey};
use crate::sys::gl::{self, Texel};

impl Gpu {
    pub(crate) fn serve(&mut self, side: u32) {
        self.strict = true;
        self.letters = Some(Letters { atlas: self.new_atlas(Texel::R8, side, side), known: HashMap::new(), keys: [RandomState::new(), RandomState::new()] });
    }

    fn new_atlas(&self, texel: Texel, side: u32, max: u32) -> Atlas {
        Atlas { tex: self.gl.texture(texel, side, side), texel, shelves: Vec::new(), top: 0, side, max }
    }

    pub(crate) fn store(&self, side: u32, max: u32) -> Store {
        Store {
            masks: self.new_atlas(Texel::R8, side, max), pictures: self.new_atlas(Texel::Rgba8, side, max),
            glyphs: HashMap::new(), shapes: HashMap::new(), paths: HashMap::new(), shades: HashMap::new(), images: HashMap::new(),
            held: Vec::new(), shared_bytes: 0, levels: Vec::new(),
        }
    }

    pub(crate) fn free_store(&self, s: Store) {
        self.gl.delete_texture(s.masks.tex);
        self.gl.delete_texture(s.pictures.tex);
        s.held.iter().for_each(|tex| self.gl.delete_texture(*tex));
        for l in s.levels {
            if let Some((t, ..)) = l.layer {
                self.free(t);
            }
        }
    }

    pub(crate) fn swap_store(&mut self, s: &mut Store) {
        use std::mem::swap;
        swap(&mut self.masks, &mut s.masks);
        swap(&mut self.pictures, &mut s.pictures);
        swap(&mut self.glyphs, &mut s.glyphs);
        swap(&mut self.shapes, &mut s.shapes);
        swap(&mut self.paths, &mut s.paths);
        swap(&mut self.shades, &mut s.shades);
        swap(&mut self.images, &mut s.images);
        swap(&mut self.held, &mut s.held);
        swap(&mut self.shared_bytes, &mut s.shared_bytes);
        swap(&mut self.levels, &mut s.levels);
    }

    pub(crate) fn empty_store(&mut self) {
        self.empty(Texel::R8);
        self.empty(Texel::Rgba8);
        self.glyphs.clear();
        for tex in std::mem::take(&mut self.held) {
            self.gl.delete_texture(tex);
        }
    }

    pub(crate) fn forget_shades(&mut self) {
        self.shades.clear();
    }

    #[cfg(test)]
    pub(crate) fn shared_letters(&self) -> usize {
        self.letters.as_ref().map_or(0, |l| l.known.len())
    }

    pub(crate) fn all_gone(&mut self) {
        if let Some(l) = &mut self.letters {
            l.known.clear();
            l.atlas.shelves.clear();
            l.atlas.top = 0;
        }
        for t in std::mem::take(&mut self.pool) {
            self.free(t);
        }
        if let Some(s) = self.island.take() {
            self.gl.delete_texture(s.tex);
        }
    }

    fn room_in(&mut self, texel: Texel, w: u32, h: u32) -> Option<(u32, u32)> {
        let mut at = self.atlas(texel).take(w, h);
        while at.is_none() && self.atlas(texel).side < self.atlas(texel).max && self.grow(texel) {
            at = self.atlas(texel).take(w, h);
        }
        at
    }

    pub(crate) fn letter(&mut self, key: MaskKey, mask: &Mask, hollow: Rc<Mask>) -> bool {
        let (_, _, w, h, cover) = mask.parts();
        if w == 0 || h == 0 {
            return true;
        }
        let (tw, th) = (w as u32, h as u32);
        if let Some(l) = &mut self.letters {
            let name = [l.keys[0].hash_one((w, h, cover)), l.keys[1].hash_one((w, h, cover))];
            if let Some(s) = l.known.get(&name) {
                self.glyphs.insert(key, (hollow, *s));
                return true;
            }
            if self.shared_bytes + cover.len() <= SHARE_MOST {
                if let Some((u, v)) = l.atlas.take(tw, th) {
                    let s = Spot { u, v, w: tw, h: th, tex: l.atlas.tex };
                    l.known.insert(name, s);
                    self.shared_bytes += cover.len();
                    self.gl.upload(s.tex, Texel::R8, (u, v, tw, th), cover, w);
                    self.glyphs.insert(key, (hollow, s));
                    return true;
                }
            }
        }
        let Some((u, v)) = self.room_in(Texel::R8, tw, th) else { return false };
        let tex = self.masks.tex;
        self.gl.upload(tex, Texel::R8, (u, v, tw, th), cover, w);
        self.glyphs.insert(key, (hollow, Spot { u, v, w: tw, h: th, tex }));
        true
    }

    pub(crate) fn picture(&mut self, id: u64, pix: &Pixmap) {
        let (w, h) = (pix.width(), pix.height());
        let (tex, u, v) = match self.room_in(Texel::Rgba8, w, h) {
            Some((u, v)) => (self.pictures.tex, u, v),
            None => {
                let tex = self.gl.texture(Texel::Rgba8, w, h);
                self.held.push(tex);
                (tex, 0, 0)
            }
        };
        self.gl.upload(tex, Texel::Rgba8, (u, v, w, h), pix.data(), w as usize);
        self.images.insert(id, Spot { u, v, w, h, tex });
    }

    pub(crate) fn clear_target(&self, t: &Target) {
        self.gl.bind_framebuffer(gl::GL_FRAMEBUFFER, t.fbo);
        self.gl.disable(gl::GL_SCISSOR_TEST);
        self.gl.clear_colour(0.0, 0.0, 0.0, 0.0);
        self.gl.clear(gl::GL_COLOR_BUFFER_BIT);
        self.gl.enable(gl::GL_SCISSOR_TEST);
    }
}

#[cfg(test)]
impl Gpu {
    pub(crate) fn small_atlases(&mut self, side: u32) {
        let side = side.min(SIDE);
        for a in [&mut self.masks, &mut self.pictures] {
            self.gl.delete_texture(a.tex);
            *a = Atlas { tex: self.gl.texture(a.texel, side, side), side, max: side, texel: a.texel, shelves: Vec::new(), top: 0 };
        }
        self.bound = (self.masks.tex, self.pictures.tex);
        self.bind_all();
        self.empty(Texel::R8);
        self.empty(Texel::Rgba8);
    }
}
