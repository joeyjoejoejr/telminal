use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute, queue,
    style::{self, Color, Colorize},
    terminal::{self, ClearType},
    Result,
};
use std::io::{stdout, Write};

struct Container {
    width: Option<u16>,
    height: Option<u16>,
    background_color: Color,
    foreground_color: Color,
    text: String,
}

impl Container {
    fn draw<W>(&self, w: &mut W) -> Result<()>
    where
        W: Write,
    {
        for row in 1..=self.height.unwrap() {
            for column in 1..=self.width.unwrap() {
                queue!(
                    w,
                    cursor::MoveTo(column, row),
                    style::PrintStyledContent(style::style(" ").on(self.background_color)),
                )?;
            }
        }

        queue!(
            w,
            cursor::MoveTo(1, 1),
            style::PrintStyledContent(
                style::style(&self.text)
                    .with(self.foreground_color)
                    .on(self.background_color)
            ),
        )?;
        Ok(())
    }
}

fn main() -> Result<()> {
    let mut stdout = stdout();
    let layout = Container {
        width: Some(30),
        height: Some(10),
        background_color: Color::Red,
        foreground_color: Color::Blue,
        text: "Hello Container".to_owned(),
    };

    execute!(stdout, terminal::EnterAlternateScreen)?;
    terminal::enable_raw_mode()?;

    loop {
        queue!(
            stdout,
            style::ResetColor,
            terminal::Clear(ClearType::All),
            cursor::Hide,
            cursor::MoveTo(1, 1)
        )?;

        layout.draw(&mut stdout)?;

        queue!(
            stdout,
            cursor::MoveTo(1, 1),
            style::PrintStyledContent("Hello".green().on_red())
        )?;
        stdout.flush()?;

        let c = loop {
            if let Ok(Event::Key(KeyEvent {
                code: KeyCode::Char(c),
                ..
            })) = event::read()
            {
                break c;
            }
        };

        if c == 'q' {
            break;
        }
    }

    execute!(
        stdout,
        style::ResetColor,
        cursor::Show,
        terminal::LeaveAlternateScreen
    )?;
    Ok(())
}
