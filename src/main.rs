use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute, queue,
    style::{self, Print},
    terminal::{self, ClearType},
};
use std::error::Error;
use std::io::{stdout, Stdout, Write};

type AppResult<T> = Result<T, Box<dyn Error>>;

trait ViewNode: std::fmt::Debug {
    fn render(&self, stdout: &mut Stdout) -> AppResult<()>;
}

#[derive(Debug)]
struct View<Msg: std::fmt::Debug> {
    child: Option<Box<dyn ViewNode>>,
    on_key_press: Option<fn(KeyEvent) -> Msg>,
}

impl<Msg> ViewNode for View<Msg>
where
    Msg: std::fmt::Debug,
{
    fn render(&self, stdout: &mut Stdout) -> AppResult<()> {
        if let Some(ref child) = self.child {
            child.render(stdout)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
struct TextView {
    text: String,
}

impl ViewNode for TextView {
    fn render(&self, stdout: &mut Stdout) -> AppResult<()> {
        queue!(stdout, Print(&self.text)).map_err(|e| e.into())
    }
}

type UpdateFn<Msg, Model> = fn(Msg, &Model) -> Model;
type ViewFn<Msg, Model> = fn(&Model) -> View<Msg>;

struct Terminal<Model, Msg>
where
    Msg: std::fmt::Debug,
{
    init: Model,
    update: UpdateFn<Msg, Model>,
    view: ViewFn<Msg, Model>,
}

impl<Model, Msg> Terminal<Model, Msg>
where
    Model: Clone,
    Msg: std::fmt::Debug,
{
    fn new(init: Model, update: UpdateFn<Msg, Model>, view: ViewFn<Msg, Model>) -> AppResult<Self> {
        let mut stdout = stdout();
        execute!(stdout, terminal::EnterAlternateScreen)?;
        terminal::enable_raw_mode()?;

        Ok(Self { init, update, view })
    }

    fn run(&self) -> AppResult<()> {
        let mut stdout = stdout();
        let mut model = self.init.clone();

        loop {
            queue!(
                stdout,
                style::ResetColor,
                terminal::Clear(ClearType::All),
                style::SetForegroundColor(style::Color::White),
                style::SetBackgroundColor(style::Color::Black),
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
    Msg: std::fmt::Debug,
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

#[derive(Clone)]
struct Model(u32);

#[derive(Debug)]
enum Msg {
    None,
    Increment,
    Decrement,
}

fn update(msg: Msg, model: &Model) -> Model {
    match msg {
        Msg::Increment => Model(model.0 + 1),
        Msg::Decrement => Model(model.0 - 1),
        Msg::None => Model(model.0),
    }
}

fn view(model: &Model) -> View<Msg> {
    View {
        child: Some(Box::new(TextView {
            text: format!("{}", model.0),
        })),
        on_key_press: Some(|e: KeyEvent| match e {
            KeyEvent {
                code: KeyCode::Up, ..
            } => Msg::Increment,
            KeyEvent {
                code: KeyCode::Down,
                ..
            } => Msg::Decrement,
            _ => Msg::None,
        }),
    }
}

fn main() -> AppResult<()> {
    Terminal::new(Model(0), update, view)?.run()
}
