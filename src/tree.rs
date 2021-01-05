use super::{Result, ScreenBuffer};
use crossterm::{event::KeyEvent, style::Color};
use std::fmt::Debug;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Default, Clone, Copy)]
pub struct Bounds {
    pub origin: (u16, u16),
    pub size: (u16, u16),
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Style {
    pub color: Option<Color>,
    pub background_color: Option<Color>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ViewNode<Msg: PartialEq + Debug> {
    Column(Vec<ViewNode<Msg>>),
    Container {
        child: Box<ViewNode<Msg>>,
        style: Style,
        on_key_press: Option<fn(KeyEvent) -> Msg>,
    },
    Row(Vec<ViewNode<Msg>>),
    Text(String),
    None,
}

impl<Msg: PartialEq + Debug> Default for ViewNode<Msg> {
    fn default() -> Self {
        Self::None
    }
}

impl<Msg: PartialEq + Debug> ViewNode<Msg> {
    pub fn boxed(self) -> Box<Self> {
        Box::new(self)
    }
}

pub fn render<Msg: PartialEq + Debug>(
    view: &ViewNode<Msg>,
    screen: &mut ScreenBuffer,
    bounds: &Bounds,
) -> Result<()> {
    match view {
        ViewNode::None => Ok(()),
        ViewNode::Text(text) => render_text(text, screen, bounds),
        ViewNode::Column(column) => render_column(column, screen, bounds),
        ViewNode::Row(row) => render_row(row, screen, bounds),
        container => render_container(container, screen, bounds),
    }
}

fn render_text(text: &str, screen: &mut ScreenBuffer, bounds: &Bounds) -> Result<()> {
    let (x, y) = bounds.origin;

    for (i, char) in text.graphemes(true).enumerate() {
        let character = &mut screen[(x as usize + i, y as usize)];
        character.character = String::from(char);
    }
    Ok(())
}

fn render_column<Msg: PartialEq + Debug>(
    _column: &[ViewNode<Msg>],
    _screen: &mut ScreenBuffer,
    _bounds: &Bounds,
) -> Result<()> {
    Ok(())
}

fn render_row<Msg: PartialEq + Debug>(
    rows: &[ViewNode<Msg>],
    screen: &mut ScreenBuffer,
    bounds: &Bounds,
) -> Result<()> {
    let len = rows.len();
    let offset = bounds.size.0 / len as u16;

    for (i, row) in rows.iter().enumerate() {
        let mut origin = bounds.origin;
        let mut size = bounds.size;
        origin.0 = offset * i as u16;
        size.0 -= offset * i as u16;
        let child_bounds = Bounds { origin, size };

        render(row, screen, &child_bounds)?;
    }
    Ok(())
}

fn render_container<Msg: PartialEq + Debug>(
    container: &ViewNode<Msg>,
    screen: &mut ScreenBuffer,
    bounds: &Bounds,
) -> Result<()> {
    if let ViewNode::Container { child, style, .. } = container {
        let foreground_color = style.color;
        let background_color = style.background_color;
        let (origin_x, origin_y) = bounds.origin;
        let (size_x, size_y) = bounds.size;

        for y in origin_y..origin_y + size_y {
            for x in origin_x..origin_x + size_x {
                let character = &mut screen[(x as usize, y as usize)];
                if let Some(c) = foreground_color {
                    character.foreground_color = c;
                }
                if let Some(c) = background_color {
                    character.background_color = c;
                }
                character.character = String::from(" ");
            }
        }
        render(child, screen, bounds)
    } else {
        Err("Unknown ViewNode".into())
    }
}
