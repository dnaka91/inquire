use std::fmt::Display;

use crossterm::style::Color;

use crate::{
    error::{InquireError, InquireResult},
    input::Input,
    key::Key,
    terminal::{Style, Terminal},
    utils::Page,
};

pub struct Renderer<'a> {
    cur_line: usize,
    terminal: Terminal<'a>,
}

pub struct Token<'a> {
    pub content: &'a str,
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub style: Option<Style>,
}

impl<'a> Token<'a> {
    pub fn new(content: &'a str) -> Self {
        Self {
            content,
            fg: None,
            bg: None,
            style: None,
        }
    }

    #[allow(unused)]
    pub fn empty() -> Self {
        Self::new("")
    }

    pub fn with_fg(mut self, fg: Color) -> Self {
        self.fg = Some(fg);
        self
    }

    #[allow(unused)]
    pub fn with_bg(mut self, bg: Color) -> Self {
        self.bg = Some(bg);
        self
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = Some(style);
        self
    }

    pub fn print(&self, terminal: &mut Terminal) -> InquireResult<()> {
        if self.content.is_empty() {
            return Ok(());
        }

        if let Some(color) = self.fg {
            terminal.set_fg_color(color)?;
        }
        if let Some(color) = self.bg {
            terminal.set_bg_color(color)?;
        }
        if let Some(style) = &self.style {
            terminal.set_style(*style)?;
        }

        terminal.write(self.content)?;

        if let Some(_) = self.fg.as_ref() {
            terminal.reset_fg_color()?;
        }
        if let Some(_) = self.bg.as_ref() {
            terminal.reset_bg_color()?;
        }
        if let Some(_) = &self.style {
            terminal.reset_style()?;
        }

        Ok(())
    }
}

impl<'a> Renderer<'a> {
    pub fn new(terminal: Terminal<'a>) -> InquireResult<Self> {
        let mut renderer = Self {
            cur_line: 0,
            terminal,
        };

        renderer.terminal.cursor_hide()?;

        Ok(renderer)
    }

    pub fn reset_prompt(&mut self) -> InquireResult<()> {
        for _ in 0..self.cur_line {
            self.terminal.cursor_up()?;
            self.terminal.cursor_horizontal_reset()?;
            self.terminal.clear_current_line()?;
        }

        self.cur_line = 0;
        Ok(())
    }

    pub fn print_tokens(&mut self, tokens: &[Token]) -> InquireResult<()> {
        for t in tokens {
            t.print(&mut self.terminal)?;
        }

        Ok(())
    }

    pub fn print_error_message(&mut self, message: &str) -> InquireResult<()> {
        Token::new(&format!("# {}", message))
            .with_fg(Color::Red)
            .print(&mut self.terminal)?;

        self.new_line()?;

        Ok(())
    }

    pub fn print_prompt_answer(&mut self, prompt: &str, answer: &str) -> InquireResult<()> {
        self.print_tokens(&vec![
            Token::new("? ").with_fg(Color::Green),
            Token::new(prompt),
            Token::new(&format!(" {}", answer)).with_fg(Color::Cyan),
        ])?;
        self.new_line()?;

        Ok(())
    }

    pub fn print_prompt(
        &mut self,
        prompt: &str,
        default: Option<&str>,
        content: Option<&str>,
    ) -> InquireResult<()> {
        Token::new("? ")
            .with_fg(Color::Green)
            .print(&mut self.terminal)?;
        Token::new(prompt).print(&mut self.terminal)?;

        if let Some(default) = default {
            Token::new(&format!(" ({})", default)).print(&mut self.terminal)?;
        }

        match content {
            Some(content) if !content.is_empty() => Token::new(&format!(" {}", content))
                .with_style(Style::Bold)
                .print(&mut self.terminal)?,
            _ => {}
        }

        self.new_line()?;

        Ok(())
    }

    pub fn print_prompt_input(
        &mut self,
        prompt: &str,
        default: Option<&str>,
        content: &Input,
    ) -> InquireResult<()> {
        Token::new("? ")
            .with_fg(Color::Green)
            .print(&mut self.terminal)?;
        Token::new(prompt).print(&mut self.terminal)?;

        if let Some(default) = default {
            Token::new(&format!(" ({})", default)).print(&mut self.terminal)?;
        }

        let (before, mut at, after) = content.split();

        if at.is_empty() {
            at.push(' ');
        }

        self.print_tokens(&[
            Token::new(" "),
            Token::new(&before),
            Token::new(&at).with_bg(Color::Grey).with_fg(Color::Black),
            Token::new(&after),
        ])?;

        self.new_line()?;

        Ok(())
    }

    pub fn print_help(&mut self, message: &str) -> InquireResult<()> {
        Token::new(&format!("[{}]", message))
            .with_fg(Color::Cyan)
            .print(&mut self.terminal)?;
        self.new_line()?;

        Ok(())
    }

    pub fn print_option(&mut self, cursor: bool, content: &str) -> InquireResult<()> {
        match cursor {
            true => Token::new(&format!("> {}", content))
                .with_fg(Color::Cyan)
                .print(&mut self.terminal),
            false => Token::new(&format!("  {}", content)).print(&mut self.terminal),
        }?;

        self.new_line()?;

        Ok(())
    }

    pub fn print_options<T>(&mut self, page: Page<T>) -> InquireResult<()>
    where
        T: Display,
    {
        let length = page.content.len();
        for (idx, option) in page.content.iter().enumerate() {
            let (c, color) = if idx == 0 && !page.first {
                ("^ ", Color::Reset)
            } else if (idx + 1) == length && !page.last {
                ("v ", Color::Reset)
            } else if idx == page.selection {
                (" >", Color::Cyan)
            } else {
                ("  ", Color::Reset)
            };

            Token::new(&format!("{} {}", c, option))
                .with_fg(color)
                .print(&mut self.terminal)?;

            self.new_line()?;
        }

        Ok(())
    }

    pub fn print_multi_option(
        &mut self,
        cursor: bool,
        checked: bool,
        content: &str,
    ) -> InquireResult<()> {
        self.print_tokens(&vec![
            match cursor {
                true => Token::new("> ").with_fg(Color::Cyan),
                false => Token::new("  "),
            },
            match checked {
                true => Token::new("[x] ").with_fg(Color::Green),
                false => Token::new("[ ] "),
            },
            Token::new(content),
        ])?;

        self.new_line()?;

        Ok(())
    }

    #[cfg(feature = "date")]
    pub fn print_calendar_month(
        &mut self,
        month: chrono::Month,
        year: i32,
        week_start: chrono::Weekday,
        today: chrono::NaiveDate,
        selected_date: chrono::NaiveDate,
        min_date: Option<chrono::NaiveDate>,
        max_date: Option<chrono::NaiveDate>,
    ) -> InquireResult<()> {
        use crate::date_utils::get_start_date;
        use chrono::{Datelike, Duration};
        use std::ops::Sub;

        // print header (month year)
        let header = format!("{} {}", month.name().to_lowercase(), year);

        self.print_tokens(&vec![
            Token::new("> ").with_fg(Color::Green),
            Token::new(&format!("{:^20}", header)),
        ])?;
        self.new_line()?;

        // print week header
        let mut current_weekday = week_start;
        let mut week_days: Vec<String> = vec![];
        for _ in 0..7 {
            let mut formatted = format!("{}", current_weekday);
            formatted.make_ascii_lowercase();
            formatted.pop();
            week_days.push(formatted);

            current_weekday = current_weekday.succ();
        }
        let week_days = week_days.join(" ");

        Token::new("> ")
            .with_fg(Color::Green)
            .print(&mut self.terminal)?;
        self.terminal.write(&week_days)?;
        self.new_line()?;

        // print dates
        let mut date_it = get_start_date(month, year);
        // first date of week-line is possibly in the previous month
        if date_it.weekday() == week_start {
            date_it = date_it.sub(Duration::weeks(1));
        } else {
            while date_it.weekday() != week_start {
                date_it = date_it.pred();
            }
        }

        for _ in 0..6 {
            Token::new("> ")
                .with_fg(Color::Green)
                .print(&mut self.terminal)?;

            for i in 0..7 {
                if i > 0 {
                    self.terminal.write(" ")?;
                }

                let date = format!("{:2}", date_it.day());

                let mut token = Token::new(&date);

                if date_it == selected_date {
                    token = token.with_bg(Color::Grey).with_fg(Color::Black);
                } else if date_it == today {
                    token = token.with_fg(Color::Green);
                } else if date_it.month() != month.number_from_month() {
                    token = token.with_fg(Color::DarkGrey);
                }

                if let Some(min_date) = min_date {
                    if date_it < min_date {
                        token = token.with_fg(Color::DarkGrey);
                    }
                }

                if let Some(max_date) = max_date {
                    if date_it > max_date {
                        token = token.with_fg(Color::DarkGrey);
                    }
                }

                token.print(&mut self.terminal)?;

                date_it = date_it.succ();
            }

            self.new_line()?;
        }

        Ok(())
    }

    pub fn cleanup(&mut self, message: &str, answer: &str) -> InquireResult<()> {
        self.reset_prompt()?;
        self.print_prompt_answer(message, answer)?;

        Ok(())
    }

    pub fn flush(&mut self) -> InquireResult<()> {
        self.terminal.flush()?;

        Ok(())
    }

    pub fn read_key(&mut self) -> InquireResult<Key> {
        self.terminal
            .read_key()
            .map(Key::from)
            .map_err(InquireError::from)
    }

    fn new_line(&mut self) -> InquireResult<()> {
        self.terminal.cursor_horizontal_reset()?;
        self.terminal.write("\n")?;
        self.cur_line = self.cur_line.saturating_add(1);

        Ok(())
    }
}

impl<'a> Drop for Renderer<'a> {
    fn drop(&mut self) {
        let _ = self.terminal.cursor_show();
    }
}
