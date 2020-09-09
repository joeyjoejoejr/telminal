use crossterm::{
    cursor,
    event::{Event, KeyCode, KeyEvent},
    execute, queue,
    style::{self, Print},
    terminal::{self, ClearType},
};
use std::error::Error;
use std::fmt::Debug;
use std::io::{stdout, Write};
use unicode_segmentation::UnicodeSegmentation;

pub use crossterm::event;
pub use crossterm::style::Color;

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Character {
    foreground_color: Color,
    background_color: Color,
    character: String,
}

impl Default for Character {
    fn default() -> Self {
        Self {
            foreground_color: Color::Reset,
            background_color: Color::Reset,
            character: String::from(" "),
        }
    }
}

pub trait ViewNode: Debug {
    fn render(&self, stdout: &mut Vec<Vec<Character>>, bounds: &Bounds) -> Result<()>;
    fn style(&self) -> Style {
        Style::default()
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Style {
    pub color: Option<Color>,
    pub background_color: Option<Color>,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Bounds {
    origin: (u16, u16),
    size: (u16, u16),
}

#[derive(Debug)]
pub struct View<Msg: Debug> {
    child: Option<Box<dyn ViewNode>>,
    on_key_press: Option<fn(KeyEvent) -> Msg>,
    style: Style,
}

impl<Msg> View<Msg>
where
    Msg: Debug,
{
    pub fn new() -> Self {
        View {
            child: None,
            on_key_press: None,
            style: Style::default(),
        }
    }

    pub fn child<T: ViewNode + 'static>(mut self, child: T) -> Self {
        self.child.replace(Box::new(child));
        self
    }

    pub fn on_key_press(mut self, cb: fn(KeyEvent) -> Msg) -> Self {
        self.on_key_press.replace(cb);
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl<Msg> ViewNode for View<Msg>
where
    Msg: Debug,
{
    fn render(&self, stdout: &mut Vec<Vec<Character>>, bounds: &Bounds) -> Result<()> {
        let foreground_color = self.style.color;
        let background_color = self.style.background_color;
        let (origin_x, origin_y) = bounds.origin;
        let (size_x, size_y) = bounds.size;

        for y in origin_y..origin_y + size_y {
            for x in origin_x..origin_x + size_x {
                let character = &mut stdout[y as usize][x as usize];
                foreground_color.map(|c| character.foreground_color = c);
                background_color.map(|c| character.background_color = c);
                character.character = String::from(" ");
            }
        }

        if let Some(ref child) = self.child {
            child.render(stdout, bounds)?;
        }
        Ok(())
    }

    fn style(&self) -> Style {
        self.style
    }
}

#[derive(Debug)]
pub struct TextView {
    pub text: String,
}

impl TextView {
    pub fn new(text: String) -> Self {
        Self { text }
    }
}

impl ViewNode for TextView {
    fn render(&self, stdout: &mut Vec<Vec<Character>>, bounds: &Bounds) -> Result<()> {
        let (x, y) = bounds.origin;

        for (i, char) in self.text.graphemes(true).enumerate() {
            let character = &mut stdout[y as usize][x as usize + i];
            character.character = String::from(char);
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct RowView {
    children: Vec<Box<dyn ViewNode>>,
}

impl RowView {
    pub fn new(children: Vec<Box<dyn ViewNode>>) -> Self {
        Self { children }
    }
}

impl ViewNode for RowView {
    fn render(&self, stdout: &mut Vec<Vec<Character>>, bounds: &Bounds) -> Result<()> {
        let len = self.children.len();
        let offset = bounds.size.0 / len as u16;

        for (i, child) in self.children.iter().enumerate() {
            let mut origin = bounds.origin;
            let mut size = bounds.size;
            origin.0 = offset * i as u16;
            size.0 = size.0 - offset * i as u16;
            let child_bounds = Bounds { origin, size };
            child.render(stdout, &child_bounds)?;
        }

        Ok(())
    }
}

type UpdateFn<Msg, Model> = fn(Msg, &Model) -> Model;
type ViewFn<Msg, Model> = fn(&Model) -> View<Msg>;

pub struct Terminal<Model, Msg>
where
    Msg: Debug,
{
    init: Model,
    update: UpdateFn<Msg, Model>,
    view: ViewFn<Msg, Model>,
    size: (u16, u16),
}

impl<Model, Msg> Terminal<Model, Msg>
where
    Model: Clone,
    Msg: Debug,
{
    pub fn new(
        init: Model,
        update: UpdateFn<Msg, Model>,
        view: ViewFn<Msg, Model>,
    ) -> Result<Self> {
        let mut stdout = stdout();
        execute!(stdout, terminal::EnterAlternateScreen)?;
        terminal::enable_raw_mode()?;
        let size = terminal::size()?;

        Ok(Self {
            init,
            update,
            view,
            size,
        })
    }

    pub fn run(&self) -> Result<()> {
        let mut stdout = stdout();
        let mut model = self.init.clone();
        let mut old_buffer: Vec<Vec<Character>> =
            vec![vec![Character::default(); self.size.0 as usize]; self.size.1 as usize];

        queue!(
            stdout,
            style::ResetColor,
            terminal::Clear(ClearType::All),
            cursor::Hide,
            cursor::MoveTo(1, 1),
        )?;

        loop {
            let view = (self.view)(&model);
            let mut new_buffer: Vec<Vec<Character>> =
                vec![vec![Character::default(); self.size.0 as usize]; self.size.1 as usize];

            view.render(
                &mut new_buffer,
                &Bounds {
                    origin: (0, 0),
                    size: self.size,
                },
            )?;

            for (i, (new, old)) in new_buffer
                .iter()
                .flatten()
                .zip(old_buffer.iter().flatten())
                .enumerate()
            {
                if new != old {
                    let y = i as u16 / self.size.0;
                    let x = i as u16 % self.size.0;
                    queue!(
                        stdout,
                        cursor::MoveTo(x, y),
                        style::SetForegroundColor(new.foreground_color),
                        style::SetBackgroundColor(new.background_color),
                        Print(&new.character)
                    )?;
                }
            }
            stdout.flush()?;

            old_buffer = new_buffer;

            match event::read()? {
                Event::Key(KeyEvent {
                    code: KeyCode::Char('q'),
                    ..
                }) => break Ok(()),
                Event::Key(event) => {
                    if let Some(key_press) = view.on_key_press {
                        let message = (key_press)(event);
                        model = (self.update)(message, &model);
                    }
                }
                _ => {}
            }
        }
    }
}

impl<Model, Msg> Drop for Terminal<Model, Msg>
where
    Msg: Debug,
{
    fn drop(&mut self) {
        let mut stdout = stdout();
        execute!(
            stdout,
            style::ResetColor,
            cursor::Show,
            terminal::LeaveAlternateScreen
        )
        .unwrap();
    }
}
