//! Scene-node traversal and constructors, split without behavior changes.
use super::*;

impl SceneWidget {
    /// Resolve the active tab option by authored/live value. Missing or
    /// stale values deterministically fall back to the first tab/panel,
    /// mirroring jian-core's `render::scene::active_tab_index`.
    pub fn active_tab_index(&self) -> usize {
        self.value_str
            .as_deref()
            .and_then(|value| self.options.iter().position(|tab| tab.value == value))
            .unwrap_or(0)
    }
}

impl SceneNode {
    /// Children that participate in paint **and** hit-test.
    ///
    /// `tabs` is the only first-class widget whose children are alternative
    /// panels rather than ordinary descendants (`tabs[i]` maps to
    /// `children[i]`), and a tabs frame compiles to a single-cell grid where
    /// every panel overlaps the others. Painter and hit-test must therefore
    /// share this one rule: without it, clicking visible content on the second
    /// tab selects the first tab's panel, which was never drawn.
    pub fn visible_children(&self) -> &[SceneNode] {
        let Some(widget) = self.widget.as_ref().filter(|widget| widget.kind == "tabs") else {
            return &self.children;
        };
        self.children
            .get(widget.active_tab_index())
            .map(std::slice::from_ref)
            .unwrap_or_default()
    }

    /// Depth-first search for the node with `id` in this subtree
    /// (self included). Mirrors `SceneNode::find`.
    pub fn find(&self, id: &str) -> Option<&SceneNode> {
        #[cfg(any(test, feature = "test-support"))]
        record_find_visit();

        if self.id == id {
            return Some(self);
        }
        for child in &self.children {
            if let Some(found) = child.find(id) {
                return Some(found);
            }
        }
        None
    }

    pub(super) fn translate_subtree(&mut self, dx: f32, dy: f32) {
        self.bounds.origin.x += dx;
        self.bounds.origin.y += dy;
        if let Some(origin) = &mut self.css_paint_origin {
            origin.x += dx;
            origin.y += dy;
        }
        if rect_has_extent(self.aggregate_bounds_cache) {
            self.aggregate_bounds_cache.origin.x += dx;
            self.aggregate_bounds_cache.origin.y += dy;
        }
        for point in &mut self.points {
            point.x += dx;
            point.y += dy;
        }
        for anchor in &mut self.path_anchors {
            anchor.pos.x += dx;
            anchor.pos.y += dy;
            if let Some(handle) = anchor.handle_in.as_mut() {
                handle.x += dx;
                handle.y += dy;
            }
            if let Some(handle) = anchor.handle_out.as_mut() {
                handle.x += dx;
                handle.y += dy;
            }
        }
        for child in &mut self.children {
            child.translate_subtree(dx, dy);
        }
    }

    /// Resolved bounds for selection / rotation-pivot math: the node's
    /// own `bounds` when it is bounded, otherwise a loader-precomputed
    /// subtree rect when available, otherwise the union of children's
    /// aggregate bounds for hand-built scenes.
    pub fn aggregate_bounds(&self) -> Rect {
        if rect_has_extent(self.bounds) {
            return self.bounds;
        }
        if rect_has_extent(self.aggregate_bounds_cache) {
            return self.aggregate_bounds_cache;
        }
        Self::compute_aggregate_bounds(self.bounds, &self.children)
    }

    /// Compute the aggregate subtree rect from resolved own bounds and
    /// children. Used by the loader during scene construction and by
    /// [`aggregate_bounds`](Self::aggregate_bounds) as the uncached
    /// fallback for hand-built test scenes.
    pub fn compute_aggregate_bounds(bounds: Rect, children: &[SceneNode]) -> Rect {
        if rect_has_extent(bounds) {
            return bounds;
        }
        let mut iter = children
            .iter()
            .map(SceneNode::aggregate_bounds)
            .filter(|r| rect_has_extent(*r));
        let Some(first) = iter.next() else {
            return Rect::ZERO;
        };
        let (mut min_x, mut min_y) = (first.origin.x, first.origin.y);
        let (mut max_x, mut max_y) = (first.origin.x + first.size.x, first.origin.y + first.size.y);
        for r in iter {
            min_x = min_x.min(r.origin.x);
            min_y = min_y.min(r.origin.y);
            max_x = max_x.max(r.origin.x + r.size.x);
            max_y = max_y.max(r.origin.y + r.size.y);
        }
        Rect::xywh(min_x, min_y, max_x - min_x, max_y - min_y)
    }

    /// Construct a leaf render node with all paint fields cleared.
    /// Builders set `bounds` / `fill` / `text` / … after.
    pub fn leaf(id: impl Into<String>, kind: NodeKind) -> Self {
        Self {
            id: id.into(),
            kind,
            bounds: Rect::ZERO,
            css_paint_origin: None,
            text_grayscale: false,
            aggregate_bounds_cache: Rect::ZERO,
            opacity: 1.0,
            composite_opacity: 1.0,
            blend_mode: ImageBlendMode::Normal,
            rotation: 0.0,
            flip_x: false,
            flip_y: false,
            corner_radius: 0.0,
            corner_radii: None,
            clip_content: false,
            fill: None,
            fill_layers: Vec::new(),
            fill_type: SceneFillType::Solid,
            gradient: None,
            shader: None,
            stroke: None,
            text: None,
            text_runs: Vec::new(),
            font_family: String::new(),
            font_size: 0.0,
            font_weight: 0,
            italic: false,
            underline: false,
            strikethrough: false,
            line_height: 0.0,
            letter_spacing: 0.0,
            text_align: SceneTextAlign::Left,
            text_vertical_align: SceneTextVerticalAlign::Top,
            text_wrap: false,
            points: Vec::new(),
            path_anchors: Vec::new(),
            path_closed: false,
            is_mask: false,
            mask_type: None,
            even_odd_fill: false,
            svg_path: None,
            arc_start_angle: None,
            arc_sweep_angle: None,
            arc_inner_radius: None,
            polygon_sides: 3,
            image_src: None,
            video: None,
            image_src_id: 0,
            image_fit: SceneImageFit::Fill,
            image_blend_mode: ImageBlendMode::Normal,
            image_transform: None,
            image_original_size: None,
            image_tile_scale: 1.0,
            image_adjustments: ImageAdjustments::default(),
            effects: Vec::new(),
            hidden: false,
            locked: false,
            widget: None,
            children: Vec::new(),
        }
    }
}
