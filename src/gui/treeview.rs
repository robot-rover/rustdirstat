use std::iter;

use iced::{
    advanced::{
        graphics::text::cosmic_text::Align, layout, renderer, text::{self, Paragraph}, widget::tree, Text, Widget
    }, alignment, widget::text::{LineHeight, Shaping}, Alignment, Color, Element, Length, Point, Rectangle, Size
};

pub struct TreeViewData {
    // cols_cache: Option<Box<[String]>>,
    expanded: bool,
}

impl Default for TreeViewData {
    fn default() -> Self {
        TreeViewData {
            // cols_cache: None,
            expanded: false,
        }
    }
}

pub trait TreeWalk: Sized {
    const N_COLS: usize;
    fn children(&self) -> impl Iterator<Item = Self>;
    fn to_cols(&self) -> Vec<String>;
    // fn get_data(&self) -> &TreeViewData;
    // fn get_data_mut(&mut self) -> &mut TreeViewData;
}

impl TreeWalk for () {
    const N_COLS: usize = 0;

    fn children(&self) -> impl Iterator<Item = Self> {
        iter::empty()
    }

    fn to_cols(&self) -> Vec<String> {
        Vec::new()
    }
}

pub struct TreeView<T: TreeWalk> {
    tree: T,
    bounds: Size,
}

impl<T: TreeWalk> TreeView<T> {
    pub fn new(tree: T) -> Self {
        TreeView {
            tree,
            bounds: Size::ZERO,
        }
    }
}

#[derive(Debug)]
pub struct TreeViewState<P: Paragraph> {
    col_widths: Vec<f32>,
    top_offset: f32,
    row_text: Vec<Vec<String>>,
    row_para: Vec<Vec<P>>,
}

impl<Message, Theme, Renderer, T> Widget<Message, Theme, Renderer> for TreeView<T>
where
    Renderer: text::Renderer,
    T: TreeWalk,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<TreeViewState<Renderer::Paragraph>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(TreeViewState::<Renderer::Paragraph> {
            col_widths: vec![200.0; T::N_COLS],
            top_offset: 0.0,
            row_text: Vec::new(),
            row_para: Vec::new(),
        })
    }

    fn size(&self) -> iced::Size<iced::Length> {
        Size {
            width: Length::Shrink,
            height: Length::Shrink,
        }
    }

    fn layout(
        &self,
        tree: &mut iced::advanced::widget::Tree,
        renderer: &Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        println!("Layout");
        let size = renderer.default_size();
        let line_height = LineHeight::default();
        let state = tree
            .state
            .downcast_mut::<TreeViewState<Renderer::Paragraph>>();

        state.row_text = self.tree.children().map(|row| row.to_cols()).collect();
        println!("row_text: {:?}", &state.row_text);
        state.row_para = state.row_text.iter().map(|row| {
            row.iter().zip(&state.col_widths).map(|(col, &width)| {
                let mut para = Renderer::Paragraph::default();
                para.update(Text {
                    content: &col,
                    bounds: Size::new(width, line_height.to_absolute(size).0),
                    size,
                    line_height,
                    font: renderer.default_font(),
                    horizontal_alignment: alignment::Horizontal::Left,
                    vertical_alignment: alignment::Vertical::Top,
                    shaping: Shaping::Basic,
                });
                para
            }).collect()
        }).collect();

        let width = Length::Fill;
        let height = Length::Fill;
        layout::sized(limits, width, height, |limits| {
            println!("{:?}", limits);
            limits.min()
        })
    }

    fn draw(
        &self,
        tree: &iced::advanced::widget::Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
        _cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        println!("Draw");
        let state = tree
            .state
            .downcast_ref::<TreeViewState<Renderer::Paragraph>>();
        let line_height = 20.0; // TODO
        let bounds = layout.bounds(); // TODO: also clip the viewport?
        // println!("bounds: {:?}, viewport: {:?}", bounds, viewport);
        for (idx, row) in state.row_para.iter().enumerate() {
            let y = state.top_offset + idx as f32 * line_height;
            let mut x = 0.0;
            for (para, &width) in row.iter().zip(&state.col_widths) {
                let top_left = Point::new(bounds.x + x, bounds.y + y);
                let size = Size::new(width, line_height);
                let clip = Rectangle::new(top_left, size).intersection(&layout.bounds());
                // println!("clip: {:?}\n  tl: {:?}\n  sz: {:?}\n  vp: {:?}\n  bn: {:?}", clip, top_left, size, viewport, layout.bounds());
                if let Some(clip_some) = clip {
                    renderer.fill_paragraph(&para, top_left, Color::BLACK, clip_some);
                }
                x += width;
            }
        }
    }
}

impl<'a, Message, Theme, Renderer, T> From<TreeView<T>> for Element<'a, Message, Theme, Renderer>
where
    Renderer: text::Renderer,
    T: TreeWalk + 'a,
{
    fn from(tree_view: TreeView<T>) -> Self {
        Self::new(tree_view)
    }
}
