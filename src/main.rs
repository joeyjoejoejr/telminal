use telminal::{
    event::{KeyCode, KeyEvent},
    Color, Result, Style, Terminal, TextView, View,
};

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
    View::new()
        .style(Style {
            color: Some(Color::Red),
            background_color: Some(Color::Blue),
            ..Default::default()
        })
        .child(TextView::new(format!("{}", model.0)))
        .on_key_press(|e| match e {
            KeyEvent {
                code: KeyCode::Up, ..
            } => Msg::Increment,
            KeyEvent {
                code: KeyCode::Down,
                ..
            } => Msg::Decrement,
            _ => Msg::None,
        })
}

fn main() -> Result<()> {
    Terminal::new(Model(0), update, view)?.run()
}
