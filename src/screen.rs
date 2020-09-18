use crossterm::style::Color;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Character {
    pub foreground_color: Color,
    pub background_color: Color,
    pub character: String,
}

impl Default for Character {
    fn default() -> Self {
        Self {
            foreground_color: Color::Reset,
            background_color: Color::Reset,
            character: String::from(" "),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScreenBuffer {
    width: usize,
    height: usize,
    data: Vec<Character>,
}

impl std::ops::Index<(usize, usize)> for ScreenBuffer {
    type Output = Character;

    fn index(&self, idx: (usize, usize)) -> &Self::Output {
        let (col, row) = idx;
        assert!(col < self.width);
        assert!(row < self.height);
        &self.data[row * self.width + col]
    }
}

impl std::ops::IndexMut<(usize, usize)> for ScreenBuffer {
    fn index_mut(&mut self, idx: (usize, usize)) -> &mut Self::Output {
        let (col, row) = idx;
        assert!(col < self.width);
        assert!(row < self.height);
        &mut self.data[row * self.width + col]
    }
}

impl ScreenBuffer {
    pub fn new(width: usize, height: usize, default: Character) -> Self {
        Self {
            width,
            height,
            data: vec![default; width * height],
        }
    }

    pub fn iter(&self) -> std::slice::Iter<Character> {
        self.data.iter()
    }
}
