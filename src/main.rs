use telminal::{
    event::{KeyCode, KeyEvent},
    Color, Result, RowView, Style, Terminal, TextView, View,
};

#[derive(Clone)]
struct Model(u32);

#[derive(Debug)]
enum Msg {
    None,
    KeyPressed(KeyEvent),
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
        Msg::KeyPressed(_) | Msg::None => Model(model.0),
    }
}

fn view(model: &Model) -> View<Msg> {
    View::new()
        .style(Style {
            color: Some(Color::Blue),
            background_color: Some(Color::White),
            ..Default::default()
        })
        .child(RowView::new(vec![
            Box::new(View::<Msg>::new().style(Style {
                background_color: Some(Color::Red),
                ..Default::default()
            })),
            Box::new(
                View::<Msg>::new()
                    .style(Style {
                        color: Some(Color::White),
                        background_color: Some(Color::Green),
                        ..Default::default()
                    })
                    .child(TextView::new(format!("{}", model.0))),
            ),
            Box::new(
                View::<Msg>::new()
                    .style(Style {
                        color: Some(Color::Red),
                        background_color: Some(Color::Blue),
                        ..Default::default()
                    })
                    .child(TextView::new(format!("{}", model.0))),
            ),
        ]))
        .on_key_press(Msg::KeyPressed)
}

fn main() -> Result<()> {
    Terminal::new(Model(0), update, view)?.run()
}
