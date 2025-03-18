mod common {
    pub mod image;
    pub mod text;
}
use common::image::{image_measure_function, ImageContext};
use common::text::{text_measure_function, FontMetrics, TextContext, WritingMode, LOREM_IPSUM};
use taffy::{
    compute_cached_layout, compute_flexbox_layout, compute_grid_layout, compute_leaf_layout, compute_root_layout,
    prelude::*, Cache, Layout, Style,
};

#[derive(Debug, Copy, Clone)]
#[allow(dead_code)]
enum NodeKind {
    Flexbox,
    Grid,
    Text,
    Image,
}

struct Node {
    kind: NodeKind,
    style: Style,
    text_data: Option<TextContext>,
    image_data: Option<ImageContext>,
    cache: Cache,
    layout: Layout,
    children: Vec<Node>,
}

impl Default for Node {
    fn default() -> Self {
        Node {
            kind: NodeKind::Flexbox,
            style: Style::default(),
            text_data: None,
            image_data: None,
            cache: Cache::new(),
            layout: Layout::with_order(0),
            children: Vec::new(),
        }
    }
}

#[allow(dead_code)]
impl Node {
    pub fn new_row(style: Style) -> Node {
        Node {
            kind: NodeKind::Flexbox,
            style: Style { display: Display::Flex, flex_direction: FlexDirection::Row, ..style },
            ..Node::default()
        }
    }
    pub fn new_column(style: Style) -> Node {
        Node {
            kind: NodeKind::Flexbox,
            style: Style { display: Display::Flex, flex_direction: FlexDirection::Column, ..style },
            ..Node::default()
        }
    }
    pub fn new_grid(style: Style) -> Node {
        Node { kind: NodeKind::Grid, style: Style { display: Display::Grid, ..style }, ..Node::default() }
    }
    pub fn new_text(style: Style, text_data: TextContext) -> Node {
        Node { kind: NodeKind::Text, style, text_data: Some(text_data), ..Node::default() }
    }
    pub fn new_image(style: Style, image_data: ImageContext) -> Node {
        Node { kind: NodeKind::Image, style, image_data: Some(image_data), ..Node::default() }
    }
    pub fn append_child(&mut self, node: Node) {
        self.children.push(node);
    }

    pub fn compute_layout(&mut self, available_space: Size<AvailableSpace>) {
        compute_root_layout(self, NodeId::from(usize::MAX), available_space);
    }

    /// The methods on LayoutPartialTree need to be able to access:
    ///
    ///  - The node being laid out
    ///  - Direct children of the node being laid out
    ///
    /// Each must have an ID. For children we simply use it's index. For the node itself
    /// we use usize::MAX on the assumption that there will never be that many children.
    fn node_from_id(&self, node_id: NodeId) -> &Node {
        let idx = usize::from(node_id);
        if idx == usize::MAX {
            self
        } else {
            &self.children[idx]
        }
    }

    fn node_from_id_mut(&mut self, node_id: NodeId) -> &mut Node {
        let idx = usize::from(node_id);
        if idx == usize::MAX {
            self
        } else {
            &mut self.children[idx]
        }
    }
}

struct ChildIter(std::ops::Range<usize>);
impl Iterator for ChildIter {
    type Item = NodeId;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|idx| NodeId::from(idx))
    }
}

impl taffy::TraversePartialTree for Node {
    type ChildIter<'a> = ChildIter;

    fn child_ids(&self, _node_id: NodeId) -> Self::ChildIter<'_> {
        ChildIter(0..self.children.len())
    }

    fn child_count(&self, _node_id: NodeId) -> usize {
        self.children.len()
    }

    fn get_child_id(&self, _node_id: NodeId, index: usize) -> NodeId {
        NodeId::from(index)
    }
}

impl taffy::LayoutPartialTree for Node {
    fn get_style(&self, node_id: NodeId) -> &Style {
        &self.node_from_id(node_id).style
    }

    fn set_unrounded_layout(&mut self, node_id: NodeId, layout: &Layout) {
        self.node_from_id_mut(node_id).layout = *layout
    }

    fn get_cache_mut(&mut self, node_id: NodeId) -> &mut Cache {
        &mut self.node_from_id_mut(node_id).cache
    }

    fn compute_child_layout(&mut self, node_id: NodeId, inputs: taffy::tree::LayoutInput) -> taffy::tree::LayoutOutput {
        compute_cached_layout(self, node_id, inputs, |parent, node_id, inputs| {
            let node = parent.node_from_id_mut(node_id);
            let font_metrics = FontMetrics { char_width: 10.0, char_height: 10.0 };

            match node.kind {
                NodeKind::Flexbox => compute_flexbox_layout(node, node_id, inputs),
                NodeKind::Grid => compute_grid_layout(node, node_id, inputs),
                NodeKind::Text => compute_leaf_layout(inputs, &node.style, |known_dimensions, available_space| {
                    text_measure_function(
                        known_dimensions,
                        available_space,
                        node.text_data.as_ref().unwrap(),
                        &font_metrics,
                    )
                }),
                NodeKind::Image => compute_leaf_layout(inputs, &node.style, |known_dimensions, _available_space| {
                    image_measure_function(known_dimensions, node.image_data.as_ref().unwrap())
                }),
            }
        })
    }
}

fn main() -> Result<(), taffy::TaffyError> {
    let mut root = Node::new_column(Style::DEFAULT);

    let text_node = Node::new_text(
        Style::default(),
        TextContext { text_content: LOREM_IPSUM.into(), writing_mode: WritingMode::Horizontal },
    );
    root.append_child(text_node);

    let image_node = Node::new_image(Style::default(), ImageContext { width: 400.0, height: 300.0 });
    root.append_child(image_node);

    // Compute layout
    root.compute_layout(Size::MAX_CONTENT);

    Ok(())
}
