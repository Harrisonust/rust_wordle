use ratatui::{
    Frame,
    crossterm::event::{self, Event, KeyCode},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Clear, Paragraph, Widget, Wrap},
};

use super::word::{Tile, TileState, WORD_LEN};
use super::wordle::{ROUND, Wordle};

pub enum InputState {
    EditingGuess,
    Submit,
    Quit,
    None,
}

impl Wordle {
    pub fn handle_input(&mut self) -> InputState {
        if let Ok(Event::Key(key)) = event::read() {
            match key.code {
                KeyCode::Esc => return InputState::Quit,
                KeyCode::Tab => {
                    self.game_restart();
                }
                KeyCode::Char('?') if self.is_game_over && !self.show_word_def => {
                    self.show_word_def = true;
                }
                KeyCode::Char(ch) if !self.is_game_over => {
                    if self.current_guess.len() < WORD_LEN {
                        self.current_guess.push(ch.to_ascii_uppercase());
                    }
                    return InputState::EditingGuess;
                }
                KeyCode::Backspace if !self.is_game_over => {
                    if !self.current_guess.is_empty() {
                        self.current_guess.pop();
                    }
                    return InputState::EditingGuess;
                }
                KeyCode::Enter if !self.is_game_over => {
                    return InputState::Submit;
                }
                _ => {}
            }
        }
        InputState::None
    }

    pub fn update_screen(&self, frame: &mut Frame) {
        let [outer_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(50)])
            .margin(1)
            .areas(
                frame
                    .area()
                    .centered(Constraint::Length(50), Constraint::Length(42)),
            );

        let [inner_area] = Layout::vertical([Constraint::Fill(1)])
            .margin(1)
            .areas(outer_area);

        let [msg_area, top_area, bottom_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(2),
                Constraint::Length(28),
                Constraint::Length(7),
            ])
            .margin(2)
            .areas(outer_area);

        /* border */
        let instructions = Line::from(vec![
            " Submit ".into(),
            "<Enter>".blue().bold(),
            " New game ".into(),
            "<Tab>".blue().bold(),
            " Quit ".into(),
            "<Esc>".blue().bold(),
        ]);

        Block::bordered()
            .title("Wordle")
            .title_bottom(instructions.right_aligned())
            .border_type(BorderType::Rounded)
            .render(outer_area, frame.buffer_mut());

        /* game board */
        Block::bordered()
            .border_type(BorderType::Rounded)
            .render(top_area, frame.buffer_mut());
        let [game_board_area] = Layout::vertical([Constraint::Fill(1)])
            .margin(1)
            .areas(top_area);

        if !self.err_msg.is_empty() {
            let span = Span::styled(self.err_msg.clone(), Style::default().fg(Color::Red));
            frame.render_widget(span, msg_area);
        }

        // past guesses
        let width: u16 = 5;
        let height: u16 = 3;
        let center_x = (game_board_area.left() + game_board_area.right()) / 2;
        for (row, word) in self.guess_history.iter().enumerate() {
            for (col, tile) in word.letters.iter().enumerate() {
                let area = Rect {
                    x: (center_x as i32 - width as i32 / 2
                        + ((col as i32 - 2) * (width + 2) as i32)) as u16,
                    y: game_board_area.y + 1 + (row as i32 * (height + 1) as i32) as u16,
                    width,
                    height,
                };
                tile.render(area, frame.buffer_mut());
            }
        }

        // remaining spots
        for row in (self.round - 1)..ROUND {
            for col in 0..WORD_LEN {
                let area = Rect {
                    x: (center_x as i32 - width as i32 / 2
                        + ((col as i32 - 2) * (width + 2) as i32)) as u16,
                    y: game_board_area.y + 1 + (row as i32 * (height + 1) as i32) as u16,
                    width,
                    height,
                };
                let tile = Tile {
                    letter: ' ',
                    state: TileState::Unused,
                };
                tile.render(area, frame.buffer_mut());
            }
        }

        // current_guess guess
        for (col, ch) in self.current_guess.chars().enumerate() {
            let area = Rect {
                x: (center_x as i32 - width as i32 / 2 + ((col as i32 - 2) * (width + 2) as i32))
                    as u16,
                y: game_board_area.y
                    + 1
                    + (self.guess_history.len() as i32 * (height + 1) as i32) as u16,
                width,
                height,
            };
            let tile = Tile {
                letter: ch,
                state: TileState::Absent,
            };
            tile.render(area, frame.buffer_mut());
        }

        /* keyboard */
        let qwerty = [
            "Q W E R T Y U I O P",
            " A S D F G H J K L ",
            "  Z X C V B N M    ",
        ];
        Block::bordered()
            .border_type(BorderType::Rounded)
            .render(bottom_area, frame.buffer_mut());
        let [keyboard_area] = Layout::vertical([Constraint::Fill(1)])
            .margin(1)
            .areas(bottom_area);
        let mut lines = Vec::new();
        for row in qwerty {
            let spans = row
                .chars()
                .map(|ch| {
                    if ch == ' ' {
                        Span::raw(" ")
                    } else {
                        let color = match self.used_chars.get(&ch) {
                            Some(TileState::Correct) => Color::Green,
                            Some(TileState::Present) => Color::Yellow,
                            Some(TileState::Absent) => Color::DarkGray,
                            Some(TileState::Unused) => Color::Black,
                            _ => unreachable!(),
                        };
                        Span::raw(format!(" {ch} ")).bg(color).bold()
                    }
                })
                .collect();
            lines.push(spans);
            lines.push(Line::from(vec![]));
        }
        Paragraph::new(lines)
            .alignment(Alignment::Center)
            .render(keyboard_area, frame.buffer_mut());

        if self.is_game_over {
            let mut game_result = if self.solved {
                vec![Line::from(vec![
                    Span::raw("You won! The answer is: ").fg(Color::Green),
                    Span::raw(&self.answer).bold().fg(Color::White),
                ])]
            } else {
                vec![Line::from(vec![
                    Span::raw("You lost! The answer is: ").fg(Color::LightYellow),
                    Span::raw(&self.answer).bold().fg(Color::White),
                ])]
            };

            game_result.push(Line::from(vec![
                Span::raw("Show word definition? "),
                Span::raw("<?>").blue().bold(),
            ]));
            Paragraph::new(game_result).render(msg_area, frame.buffer_mut());
        }

        if self.show_word_def {
            Clear.render(inner_area, frame.buffer_mut());

            let mut lines = Vec::new();
            if let Some(word_defs) = self.get_word_def(&self.answer) {
                if word_defs.is_empty() {
                    lines.push(Line::from("Definition not found"));
                } else {
                    for def in word_defs {
                        lines.push(Line::from(def));
                    }
                }
            } else {
                lines.push(Line::from(
                    "Connect to the internet to get word definitions",
                ));
            }

            Paragraph::new(lines)
                .block(Block::bordered())
                .wrap(Wrap { trim: true })
                .render(inner_area, frame.buffer_mut());
        }
    }
}
