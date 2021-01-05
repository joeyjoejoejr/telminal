#![recursion_limit = "1024"]
pub use crossterm::event;
pub use crossterm::style::Color;
mod screen;
pub mod tree;

use crossterm::{
    cursor,
    event::{Event, EventStream, KeyCode, KeyEvent},
    execute, queue,
    style::{self, Print},
    terminal::{self, ClearType},
};
use futures::{future::FutureExt, select, stream::BoxStream, StreamExt};
use std::error::Error;
use std::fmt::Debug;
use std::io::{stdout, Write};

use screen::{Character, ScreenBuffer};
use tree::{render, Bounds, ViewNode};

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;
pub type Sub<Msg> = BoxStream<'static, Msg>;

pub struct Terminal<Model, View, Update, Subscription> {
    init: Model,
    update: Update,
    view: View,
    subscriptions: Subscription,
    size: (u16, u16),
}

impl<M, V, U, S> Terminal<M, V, U, S> {
    pub fn new(init: M, update: U, view: V, subscriptions: S) -> Result<Self> {
        let mut stdout = stdout();
        execute!(stdout, terminal::EnterAlternateScreen)?;
        terminal::enable_raw_mode()?;
        let size = terminal::size()?;

        Ok(Self {
            init,
            update,
            view,
            subscriptions,
            size,
        })
    }

    pub fn run<Msg>(&self) -> Result<()>
    where
        Msg: Debug + PartialEq,
        M: Clone,
        V: Fn(&M) -> ViewNode<Msg>,
        U: Fn(Msg, &M) -> M,
        S: Fn(&M) -> Sub<Msg>,
    {
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(self._run())?;
        Ok(())
    }

    async fn _run<Msg>(&self) -> Result<()>
    where
        Msg: Debug + PartialEq,
        M: Clone,
        V: Fn(&M) -> ViewNode<Msg>,
        U: Fn(Msg, &M) -> M,
        S: Fn(&M) -> Sub<Msg>,
    {
        let mut reader = EventStream::new();
        let mut subscriptions = (self.subscriptions)(&self.init);
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
            let mut event = reader.next().fuse();
            let mut sub = subscriptions.next().fuse();

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

            select! {
                maybe_event = event => {
                    if let Some(Ok(event)) = maybe_event {
                        match event {
                            Event::Key(KeyEvent { code: KeyCode::Char('q'), .. }) => break Ok(()),
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
                maybe_sub = sub => {
                    if let Some(message) = maybe_sub {
                         model = (self.update)(message, &model);
                    }
                }
            }
        }
    }
}

impl<M, V, U, S> Drop for Terminal<M, V, U, S> {
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
