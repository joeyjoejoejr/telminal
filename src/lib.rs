use crossterm::{
    cursor,
    event::{Event, KeyCode, KeyEvent},
    execute, queue,
    style::{self, Print},
    terminal::{self, ClearType},
};
use std::cell::{RefCell, RefMut};
use std::error::Error;
use std::fmt::Debug;
use std::io::{stdout, Stdout, Write};

pub use crossterm::event;
pub use crossterm::style::Color;

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub trait ViewNode: Debug {
    fn render(&self, stdout: &mut Vec<u8>, bounds: &Bounds) -> Result<()>;
    fn style(&self) -> Style {
        Style::default()
    }

    fn apply_style(&self, stdout: &mut Vec<u8>) -> Result<()> {
        let style = self.style();
        if let Some(color) = style.color {
            queue!(stdout, style::SetForegroundColor(color))?;
        }

        if let Some(color) = style.background_color {
            queue!(stdout, style::SetBackgroundColor(color))?;
        }
        Ok(())
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
    fn render(&self, stdout: &mut Vec<u8>, bounds: &Bounds) -> Result<()> {
        self.apply_style(stdout)?;

        queue!(stdout, cursor::MoveTo(bounds.origin.0, bounds.origin.1))?;

        for y in 0..bounds.size.1 {
            let line = std::iter::repeat(" ")
                .take(bounds.size.0 as usize)
                .collect::<String>();
            queue!(
                stdout,
                cursor::MoveTo(bounds.origin.0, bounds.origin.1 + y),
                Print(line),
            )?;
        }

        queue!(stdout, cursor::MoveTo(bounds.origin.0, bounds.origin.1))?;

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
    fn render(&self, stdout: &mut Vec<u8>, _: &Bounds) -> Result<()> {
        queue!(stdout, Print(&self.text)).map_err(|e| e.into())
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
    fn render(&self, stdout: &mut Vec<u8>, bounds: &Bounds) -> Result<()> {
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
    buffer: RefCell<Vec<u8>>,
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
        let buffer: Vec<u8> = vec![];

        Ok(Self {
            init,
            update,
            view,
            size,
            buffer: RefCell::new(buffer),
        })
    }

    pub fn run(&self) -> Result<()> {
        let mut stdout = stdout();
        let mut model = self.init.clone();

        loop {
            queue!(
                stdout,
                style::ResetColor,
                terminal::Clear(ClearType::All),
                cursor::Hide,
                cursor::MoveTo(1, 1),
            )?;

            let view = (self.view)(&model);

            view.render(
                &mut *self.stdout(),
                &Bounds {
                    origin: (0, 0),
                    size: self.size,
                },
            )?;
            self.stdout().flush()?;
            let mut slice = &*&self.stdout()[..];
            std::io::copy(&mut slice, &mut stdout)?;

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

    fn stdout(&self) -> RefMut<Vec<u8>> {
        self.buffer.borrow_mut()
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
