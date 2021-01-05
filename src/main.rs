use std::time::Duration;

use futures::stream::{self, StreamExt};
use futures_timer::Delay;
use telminal::{
    event::{KeyCode, KeyEvent},
    tree::{Style, ViewNode},
    Color, Result, Sub, Terminal,
};

#[derive(Clone)]
struct Model(u32);

#[derive(Debug, PartialEq)]
enum Msg {
    None,
    KeyPressed(KeyEvent),
    Tick,
}

fn update(msg: Msg, model: &Model) -> Model {
    match msg {
        Msg::KeyPressed(KeyEvent {
            code: KeyCode::Up, ..
        }) => Model(model.0 + 1),
        Msg::KeyPressed(KeyEvent {
            code: KeyCode::Down,
            ..
        }) => Model(model.0 - 1),
        Msg::Tick => Model(model.0 + 1),
        Msg::KeyPressed(_) | Msg::None => Model(model.0),
    }
}

fn view(model: &Model) -> ViewNode<Msg> {
    ViewNode::Container {
        style: Style {
            color: Some(Color::Blue),
            background_color: Some(Color::White),
            ..Default::default()
        },
        on_key_press: Some(Msg::KeyPressed),
        child: ViewNode::Row(vec![
            ViewNode::Container {
                style: Style {
                    background_color: Some(Color::Red),
                    ..Default::default()
                },
                child: ViewNode::None.boxed(),
                on_key_press: None,
            },
            ViewNode::Container {
                style: Style {
                    color: Some(Color::White),
                    background_color: Some(Color::Green),
                    ..Default::default()
                },
                child: ViewNode::Text(format!("{}", model.0)).boxed(),
                on_key_press: None,
            },
            ViewNode::Container {
                style: Style {
                    color: Some(Color::Red),
                    background_color: Some(Color::Blue),
                    ..Default::default()
                },
                child: ViewNode::Text(format!("{}", model.0)).boxed(),
                on_key_press: None,
            },
        ])
        .boxed(),
    }
}

fn subscriptions(_model: &Model) -> Sub<Msg> {
    stream::unfold((), |_| async {
        Delay::new(Duration::from_millis(1_000)).await;
        Some((Msg::Tick, ()))
    })
    .boxed()
}

fn main() -> Result<()> {
    Terminal::new(Model(0), update, view, subscriptions)?.run()
}
