use smithay::{desktop::{find_popup_root_surface, get_popup_toplevel_coords}, wayland::{seat::WaylandFocus, shell::xdg::PopupSurface}};

use crate::state;

pub fn unconstrain_popups(noctura: &state::NocturaStates, popup: &PopupSurface) {
    if let Ok(root_surf) = find_popup_root_surface(&smithay::desktop::PopupKind::Xdg(popup.clone())) {
        for w in noctura.space.elements() {
            if w.toplevel().unwrap().wl_surface() == &root_surf {
                let root_win = w;
                let output = noctura.space.outputs().next().unwrap();
                let mut output_geo = noctura.space.output_geometry(output).unwrap();
                let window_geo = noctura.space.element_geometry(root_win).unwrap();
                output_geo.loc -= get_popup_toplevel_coords(&smithay::desktop::PopupKind::Xdg(popup.clone()));
                output_geo.loc -= window_geo.loc;

                popup.with_pending_state(|state| {
                    state.geometry = state.positioner.get_unconstrained_geometry(output_geo);
                });
            }
        }
    }
}