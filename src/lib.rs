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
    fn render(&self, stdout: &mut Stdout) -> Result<()>;
    fn style(&self) -> Style {
        Style::default()
    }

    fn apply_style(&self, stdout: &mut Stdout) {
        let style = self.style();
        if let Some(color) = style.color {
            queue!(stdout, style::SetForegroundColor(color)).unwrap();
        }

        if let Some(color) = style.background_color {
            queue!(stdout, style::SetForegroundColor(color)).unwrap();
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Style {
    pub color: Option<Color>,
    pub background_color: Option<Color>,
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
    fn render(&self, stdout: &mut Stdout) -> Result<()> {
        self.apply_style(stdout);

        if let Some(ref child) = self.child {
            child.render(stdout)?;
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
    fn render(&self, stdout: &mut Stdout) -> Result<()> {
        queue!(stdout, Print(&self.text)).map_err(|e| e.into())
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
    stdout: RefCell<Stdout>,
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

        Ok(Self {
            init,
            update,
            view,
            stdout: RefCell::new(stdout),
        })
    }

    pub fn run(&self) -> Result<()> {
        let mut model = self.init.clone();

        loop {
            queue!(
                self.stdout(),
                style::ResetColor,
                terminal::Clear(ClearType::All),
                cursor::Hide,
                cursor::MoveTo(1, 1),
            )?;

            let view = (self.view)(&model);

            view.render(&mut *self.stdout())?;
            self.stdout().flush()?;

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

    fn stdout(&self) -> RefMut<Stdout> {
        self.stdout.borrow_mut()
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
