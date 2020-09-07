use crossterm::{
    cursor,
    event::{Event, KeyCode, KeyEvent},
    execute, queue,
    style::{self, Print},
    terminal::{self, ClearType},
};
use std::error::Error;
use std::fmt::Debug;
use std::io::{stdout, Stdout, Write};

pub use crossterm::event;

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub trait ViewNode: Debug {
    fn render(&self, stdout: &mut Stdout) -> Result<()>;
}

#[derive(Debug)]
pub struct View<Msg: Debug> {
    child: Option<Box<dyn ViewNode>>,
    on_key_press: Option<fn(KeyEvent) -> Msg>,
}

impl<Msg> View<Msg>
where
    Msg: Debug,
{
    pub fn new() -> Self {
        View {
            child: None,
            on_key_press: None,
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
}

impl<Msg> ViewNode for View<Msg>
where
    Msg: Debug,
{
    fn render(&self, stdout: &mut Stdout) -> Result<()> {
        if let Some(ref child) = self.child {
            child.render(stdout)?;
        }
        Ok(())
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

        Ok(Self { init, update, view })
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

            view.render(&mut stdout)?;
            stdout.flush()?;

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
