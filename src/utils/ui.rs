use ratatui::{
    Frame,
    crossterm::event::{self, Event, KeyCode},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Clear, Paragraph, Widget, Wrap},
};

use super::word::{Tile, TileState};
use super::wordle::{ROUND, Wordle};

pub enum InputState {
    Guessing,
    Submit,
    Cancel,
    GameEnd,
}

impl Wordle {
    pub fn handle_input(&mut self) -> InputState {
        if let Ok(Event::Key(key)) = event::read() {
            match key.code {
                KeyCode::Esc => return InputState::Cancel,
                KeyCode::Tab => {
                    self.game_restart();
                }
                KeyCode::Char(ch) if self.round <= 6 && !self.solved => {
                    if self.current.len() < 5 {
                        self.current.push(ch.to_ascii_uppercase());
                    }
                }
                KeyCode::Backspace if self.round <= 6 && !self.solved => {
                    if !self.current.is_empty() {
                        self.current.pop();
                    }
                }
                KeyCode::Enter if self.round <= 6 && !self.solved => {
                    return InputState::Submit;
                }
                _ => return InputState::GameEnd, // if round >= 6, ignore all input except esc
            }
        }
        InputState::Guessing
    }

    pub fn update_screen(&self, frame: &mut Frame) {
        let [outer] = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(50)])
            .margin(1)
            .areas(
                frame
                    .area()
                    .centered(Constraint::Length(50), Constraint::Length(42)),
            );

        let [inner] = Layout::vertical([Constraint::Fill(1)])
            .margin(1)
            .areas(outer);

        let [msg, top, bottom] = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(2),
                Constraint::Length(28),
                Constraint::Length(7),
            ])
            .margin(2)
            .areas(outer);

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
            .render(outer, frame.buffer_mut());

        /* game board */
        Block::bordered()
            .border_type(BorderType::Rounded)
            .render(top, frame.buffer_mut());
        let [game_board_area] = Layout::vertical([Constraint::Fill(1)]).margin(1).areas(top);

        if !self.err_msg.is_empty() {
            let span = Span::styled(self.err_msg.clone(), Style::default().fg(Color::Red));
            frame.render_widget(span, msg);
        }

        // past guesses
        let width: u16 = 5;
        let height: u16 = 3;
        let center_x = (game_board_area.left() + game_board_area.right()) / 2;
        for (row, word) in self.history.iter().enumerate() {
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
            for col in 0..5 {
                let area = Rect {
                    x: (center_x as i32 - width as i32 / 2 + ((col - 2) * (width + 2) as i32))
                        as u16,
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

        // current guess
        for (col, ch) in self.current.chars().enumerate() {
            let area = Rect {
                x: (center_x as i32 - width as i32 / 2 + ((col as i32 - 2) * (width + 2) as i32))
                    as u16,
                y: game_board_area.y + 1 + (self.history.len() as i32 * (height + 1) as i32) as u16,
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
            .render(bottom, frame.buffer_mut());
        let [keyboard_area] = Layout::vertical([Constraint::Fill(1)])
            .margin(1)
            .areas(bottom);
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
                        Span::styled(
                            format!(" {ch} "),
                            Style::default().bg(color).add_modifier(Modifier::BOLD),
                        )
                    }
                })
                .collect();
            lines.push(spans);
            lines.push(Line::from(vec![Span::default()]));
        }
        let keyboard = Paragraph::new(lines).alignment(Alignment::Center);
        frame.render_widget(keyboard, keyboard_area);

        if self.solved || self.round > ROUND {
            let game_result = if self.solved {
                Line::from(vec![
                    Span::styled(
                        "You won! The answer is: ",
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(Color::Green),
                    ),
                    Span::styled(
                        &self.answer,
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(Color::White),
                    ),
                ])
            } else {
                Line::from(vec![
                    Span::styled(
                        "You lost! The answer is: ",
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(Color::LightYellow),
                    ),
                    Span::styled(
                        &self.answer,
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(Color::White),
                    ),
                ])
            };

            frame.render_widget(Clear, inner);

            let mut lines = vec![game_result, Line::from(Span::raw(""))];
            if let Some(word_defs) = self.get_word_def(&self.answer) {
                for def in word_defs {
                    lines.push(Line::from(def));
                }
            } else {
                lines.push(Line::from(
                    "Connect to the internet to get word definitions",
                ));
            }

            let ending = Paragraph::new(lines)
                .block(Block::bordered())
                .wrap(Wrap { trim: true });

            frame.render_widget(ending, inner);
        }
    }
}
