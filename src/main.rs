use telminal::{
    event::{KeyCode, KeyEvent},
    Result, Terminal, TextView, View,
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

fn main() -> Result<()> {
    Terminal::new(Model(0), update, view)?.run()
}
