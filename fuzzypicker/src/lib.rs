//! # Fuzzypicker
//!
//! `fuzzypicker` is a Rust library for interactive fuzzy searching and selection of items in command-line applications.
//!
//! ## Features
//!
//! - Fuzzy searching of items based on user input.
//! - Interactive selection with keyboard and mouse support.
//! - Designed for integration into Rust-based command-line tools.
//!
//! ## Example
//!
//! ```rust
//! use fuzzypicker::FuzzyPicker;
//!
//! fn main() {
//!     let items = vec!["rust", "python", "javascript", "java", "c++", "go", "swift"];
//!
//!     let mut picker = FuzzyPicker::new(&items);
//!
//!     if let Ok(Some(selected_language)) = picker.pick() {
//!         println!("Selected language: {}", selected_language);
//!     } else {
//!         println!("No language selected or selection cancelled.");
//!     }
//! }
//! ```

use crossterm::{
    cursor::MoveTo,
    event::{
        poll, read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind,
        MouseButton, MouseEventKind,
    },
    style::{Print, PrintStyledContent, Stylize},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
    QueueableCommand,
};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::clone::Clone;
use std::error::Error;
use std::fmt::Display;
use std::io::{stdout, Stdout, Write};
use std::time::Duration;

/// Struct representing a fuzzy picker for interactive item selection.
pub struct FuzzyPicker<T: Display + Clone> {
    stdout: Stdout,
    matcher: SkimMatcherV2,
    items: Vec<T>,
    display_items: Vec<String>,
    num_of_items: usize,
    num_of_displayable_items: usize,
    prompt: String,
    debug: String,
    selected: usize,
    start_index: usize,
    end_index: usize,
    height: usize,
}

impl<T: Display + Clone> FuzzyPicker<T> {
    /// Constructs a new `FuzzyPicker` instance with the given list of items.
    ///
    /// # Arguments
    ///
    /// * `items` - A slice of items implementing `Display + Clone`.
    ///
    /// # Returns
    ///
    /// A new `FuzzyPicker` instance.
    pub fn new(items: &[T]) -> Self {
        let (_, h) = terminal::size().unwrap();
        let list_items = items.to_vec();
        let num_of_items = list_items.len();
        let num_of_displayable_items = num_of_items.min((h - 1) as usize);
        Self {
            stdout: stdout(),
            matcher: SkimMatcherV2::default(),
            items: list_items,
            display_items: Vec::<String>::new(),
            num_of_items,
            num_of_displayable_items,
            prompt: String::new(),
            debug: String::new(),
            selected: 0,
            start_index: 0,
            end_index: num_of_displayable_items.saturating_sub(1),
            height: h as usize,
        }
    }

    fn prev_item(&mut self) {
        if self.num_of_items <= 0 {
            return;
        }
        if self.selected == 0 {
            self.selected = self.num_of_items.saturating_sub(1);
        } else {
            self.selected = self.selected.saturating_sub(1);
        }

        if self.selected < self.start_index {
            self.start_index = self.selected;
            self.end_index = self.start_index + self.num_of_displayable_items.saturating_sub(1);
        } else if self.selected > self.end_index {
            self.end_index = self.selected;
            self.start_index = self
                .end_index
                .saturating_sub(self.num_of_displayable_items.saturating_sub(1));
        }
    }

    fn next_item(&mut self) {
        if self.num_of_items <= 0 {
            return;
        }
        self.selected = (self.selected + 1) % self.num_of_items;
        if self.selected == 0 {
            self.start_index = 0;
            self.end_index = self.num_of_displayable_items.saturating_sub(1);
        } else if self.selected > self.end_index {
            self.start_index = self.start_index.saturating_add(1);
            self.end_index = self.end_index.saturating_add(1);
        }
    }

    fn reset_scroll(&mut self) {
        self.start_index = 0;
        self.selected = self.start_index;
    }

    /// Initiates the interactive item selection process.
    ///
    /// Handles keyboard and mouse events to perform fuzzy search, selection,
    /// and navigation within the item list.
    ///
    /// # Returns
    ///
    /// `Ok(Some(selected_item))` if an item is selected,
    /// `Ok(None)` if selection is cancelled,
    /// `Err(Box<dyn Error>)` for any error encountered during selection.
    pub fn pick(&mut self) -> Result<Option<T>, Box<dyn Error>> {
        // Initialize state
        self.filter_by_prompt();
        let mut picked_item: Option<T> = None;

        // Set up terminal
        terminal::enable_raw_mode()?;
        self.stdout
            .queue(EnterAlternateScreen)?
            .queue(EnableMouseCapture)?
            .flush()?; // Add explicit flush

        // Main event loop
        let result: Result<Option<T>, Box<dyn Error>> = (|| {
            loop {
                if poll(Duration::from_millis(500))? {
                    match read()? {
                        Event::Key(event) => {
                            if event.kind == KeyEventKind::Press {
                                match event.code {
                                    KeyCode::Char(ch) => {
                                        self.prompt.push(ch);
                                        self.filter_by_prompt();
                                        self.reset_scroll();
                                    }
                                    KeyCode::Backspace => {
                                        self.prompt.pop();
                                        self.filter_by_prompt();
                                        self.reset_scroll();
                                    }
                                    KeyCode::Esc => {
                                        return Ok(None);
                                    }
                                    KeyCode::Up | KeyCode::Left => {
                                        self.prev_item();
                                    }
                                    KeyCode::Down | KeyCode::Right => {
                                        self.next_item();
                                    }
                                    KeyCode::Enter => {
                                        // Only try to get the selected item if we have items
                                        if !self.display_items.is_empty()
                                            && self.selected < self.display_items.len()
                                        {
                                            picked_item = self
                                                .items
                                                .iter()
                                                .find(|&item| {
                                                    format!("{item}")
                                                        == self.display_items[self.selected]
                                                })
                                                .cloned();
                                        }
                                        return Ok(picked_item);
                                    }
                                    _ => {}
                                }
                            }
                        }
                        Event::Mouse(event) => match event.kind {
                            MouseEventKind::Down(MouseButton::Left) => {
                                let potential_selection =
                                    (event.row.saturating_sub(1)) as usize + self.start_index;
                                if potential_selection < self.num_of_items {
                                    self.selected = potential_selection;
                                }
                            }
                            MouseEventKind::ScrollUp => {
                                if self.start_index > 0 && self.end_index > 0 {
                                    self.start_index = self.start_index.saturating_sub(2);
                                    self.end_index = self.end_index.saturating_sub(2);
                                    self.selected = self.start_index;
                                }
                            }
                            MouseEventKind::ScrollDown => {
                                if self.start_index < self.num_of_items
                                    && self.end_index + 2 < self.num_of_items
                                    && self.num_of_items > self.height - 1
                                {
                                    self.start_index += 2;
                                    self.end_index += 2;
                                    self.selected = self.start_index;
                                }
                            }
                            _ => {}
                        },
                        Event::Resize(_, rows) => {
                            self.height = rows as usize;
                            self.num_of_displayable_items = self.num_of_items.min(self.height - 1);
                            self.end_index =
                                self.start_index + self.num_of_displayable_items.saturating_sub(1);
                        }
                        _ => {}
                    }
                }
                self.render_frame()?;
            }
        })();

        // Clean up terminal state
        let cleanup_result = self.cleanup_terminal();

        // Handle potential cleanup errors
        match (result, cleanup_result) {
            (Ok(picked), Ok(())) => Ok(picked),
            (Err(e), _) => Err(e),
            (Ok(_), Err(e)) => Err(e),
        }
    }

    /// Clean up the terminal state
    fn cleanup_terminal(&mut self) -> Result<(), Box<dyn Error>> {
        self.stdout
            .queue(Clear(ClearType::All))?
            .queue(LeaveAlternateScreen)?
            .queue(DisableMouseCapture)?
            .queue(MoveTo(0, 0))?
            .flush()?;

        terminal::disable_raw_mode()?;

        // Clear the current line to prevent overlapping
        print!("\r\x1b[K");
        stdout().flush()?;

        Ok(())
    }

    fn filter_by_prompt(&mut self) {
        self.display_items = self
            .items
            .iter()
            .filter_map(|item| {
                let display_str = format!("{}", item);
                if self.prompt.is_empty()
                    || self
                        .matcher
                        .fuzzy_match(&display_str.to_lowercase(), &self.prompt.to_lowercase())
                        .unwrap_or_default()
                        != 0
                {
                    Some(display_str)
                } else {
                    None
                }
            })
            .collect();

        self.display_items.sort_by_key(|item| {
            -self
                .matcher
                .fuzzy_match(&item.to_lowercase(), &self.prompt.to_lowercase())
                .unwrap_or_default()
        });
        self.num_of_items = self.display_items.len();
        self.num_of_displayable_items = self.num_of_items.min(self.height - 1);
        if self.num_of_displayable_items == 0 {
            self.end_index = 0;
        } else {
            self.end_index = self.num_of_displayable_items - 1;
        }
    }

    fn render_frame(&mut self) -> Result<(), Box<dyn Error>> {
        let prompt_styled = format!("> {}", self.prompt).green().bold();
        let debug_info = format!("{}", self.debug).red().bold();

        self.stdout
            .queue(Clear(ClearType::All))?
            .queue(MoveTo(0, 0))?
            .queue(PrintStyledContent(prompt_styled))?;

        if !self.debug.is_empty() {
            self.stdout
                .queue(MoveTo(20, 0))?
                .queue(PrintStyledContent(debug_info))?;
        }

        let mut row = 1;
        for (index, item) in self
            .display_items
            .iter()
            .enumerate()
            .skip(self.start_index)
            .take(self.num_of_displayable_items)
        {
            self.stdout
                .queue(MoveTo(0, row))?
                .queue(PrintStyledContent(" ".on_dark_grey()))?;

            if index == self.selected {
                self.stdout
                    .queue(PrintStyledContent(" ".on_dark_grey()))?
                    .queue(PrintStyledContent(item.as_str().white().on_dark_grey()))?;
            } else {
                self.stdout.queue(Print(format!(" {}", item)))?;
            }

            row += 1;
        }

        self.stdout
            .queue(MoveTo(self.prompt.len() as u16 + 2, 0))?
            .flush()?;

        Ok(())
    }
}
