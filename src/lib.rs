pub use crossterm::event;
pub use crossterm::style::Color;
mod screen;
pub mod tree;

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

use screen::{Character, ScreenBuffer};
use tree::{render, Bounds, ViewNode};

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

type UpdateFn<Msg, Model> = fn(Msg, &Model) -> Model;
type ViewFn<Msg, Model> = fn(&Model) -> ViewNode<Msg>;

pub struct Terminal<Model, Msg>
where
    Msg: Debug + PartialEq,
{
    init: Model,
    update: UpdateFn<Msg, Model>,
    view: ViewFn<Msg, Model>,
    size: (u16, u16),
}

impl<Model, Msg> Terminal<Model, Msg>
where
    Model: Clone,
    Msg: Debug + PartialEq,
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
        let mut old_buffer = ScreenBuffer::new(
            self.size.0 as usize,
            self.size.1 as usize,
            Character::default(),
        );
        let bounds = Bounds {
            origin: (0, 0),
            size: self.size,
        };

        queue!(
            stdout,
            style::ResetColor,
            terminal::Clear(ClearType::All),
            cursor::Hide,
            cursor::MoveTo(1, 1),
        )?;

        loop {
            let view = (self.view)(&model);
            let mut new_buffer = old_buffer.clone();

            render(&view, &mut new_buffer, &bounds)?;

            for (i, (new, old)) in new_buffer.iter().zip(old_buffer.iter()).enumerate() {
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
                    if let ViewNode::Container {
                        on_key_press: Some(key_press),
                        ..
                    } = view
                    {
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
    Msg: Debug + PartialEq,
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
