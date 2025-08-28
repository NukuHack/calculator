use iced::{
	widget::{button, column, container, row, scrollable, text, text_input, Space},
	Element, Length, Sandbox, Settings, Theme, Size, Alignment, alignment::{Vertical, Horizontal},
};
mod number;
use crate::number::{BigNumber, parse_number};

#[derive(Debug, Clone)]
pub struct HistoryEntry {
	pub input: String,
	pub output: String,
	pub is_error: bool,
}

#[derive(Debug, Clone)]
pub struct Calculator {
	current_input: String,
	previous_result: Option<String>,
	history: Vec<HistoryEntry>,
	show_history: bool,
	history_index: usize, // For navigation through history
}

#[derive(Debug, Clone)]
pub enum Message {
	InputChanged(String),
	Calculate,
	Clear,
	Backspace,
	AddDigit(char),
	AddOperator(char),
	AddDecimal,
	AddScientificE,
	ToggleSign,
	ToggleHistory,
	ClearHistory,
	NavigateHistoryPrevious,
	NavigateHistoryNext,
}

impl Sandbox for Calculator {
	type Message = Message;

	fn new() -> Self {
		Calculator {
			current_input: "0".to_string(),
			previous_result: None,
			history: Vec::new(),
			show_history: false,
			history_index: 0,
		}
	}

	fn title(&self) -> String {
		String::from("Big Number Calculator Pro")
	}

	fn update(&mut self, message: Message) {
		match message {
			Message::InputChanged(input) => {
				self.current_input = input;
			}
			Message::Calculate => {
				let input = self.current_input.clone();
				match evaluate_expression(&input) {
					Ok(result) => {
						self.add_to_history(input, result.clone(), false);
						self.previous_result = Some(result.clone());
						self.current_input = result;
						self.history_index = 0; // Reset to most recent
					}
					Err(e) => {
						self.add_to_history(input, e.clone(), true);
						self.current_input = format!("Error: {}", e);
						self.history_index = 0; // Reset to most recent
					}
				}
			}
			Message::Clear => {
				self.current_input = "0".to_string();
				self.previous_result = None;
				self.history_index = 0;
			}
			Message::Backspace => {
				if self.current_input.len() > 1 && !self.current_input.starts_with("Error:") {
					self.current_input = self.current_input[..self.current_input.len() - 1].to_string();
				} else {
					self.current_input = "0".to_string();
				}
			}
			Message::AddDigit(digit) => {
				if self.current_input.starts_with("Error:") || self.current_input == "0" {
					self.current_input = digit.to_string();
				} else {
					self.current_input.push(digit);
				}
			}
			Message::AddOperator(op) => {
				if self.current_input.starts_with("Error:") {
					self.current_input = format!("0 {} ", op);
				} else {
					// Check if the last character is already an operator
					let trimmed = self.current_input.trim_end();
					if trimmed.ends_with(&['+', '-', '*', '/', '^']) {
						// Replace the last operator
						let mut chars: Vec<char> = trimmed.chars().collect();
						while let Some(&last) = chars.last() {
							if "+-*/^".contains(last) || last == ' ' {
								chars.pop();
							} else {
								break;
							}
						}
						let base: String = chars.into_iter().collect();
						self.current_input = format!("{} {} ", base, op);
					} else {
						self.current_input = format!("{} {} ", self.current_input, op);
					}
				}
			}
			Message::AddDecimal => {
				if self.current_input.starts_with("Error:") {
					self.current_input = "0.".to_string();
				} else {
					// Check if current number already has a decimal point
					let parts: Vec<&str> = self.current_input.split_whitespace().collect();
					if let Some(last_part) = parts.last() {
						if !last_part.contains('.') && !last_part.contains('e') && !last_part.contains('E') {
							if last_part.chars().all(|c| "+-*/^".contains(c)) {
								self.current_input = format!("{}0.", self.current_input);
							} else {
								self.current_input.push('.');
							}
						}
					}
				}
			}
			Message::AddScientificE => {
				if self.current_input.starts_with("Error:") {
					self.current_input = "0e".to_string();
				} else {
					// Add 'e' for scientific notation
					let parts: Vec<&str> = self.current_input.split_whitespace().collect();
					if let Some(last_part) = parts.last() {
						if !last_part.contains('e') && !last_part.contains('E') {
							if last_part.chars().all(|c| "+-*/^".contains(c)) {
								self.current_input = format!("{}1e", self.current_input);
							} else {
								self.current_input.push('e');
							}
						}
					}
				}
			}
			Message::ToggleSign => {
				if self.current_input.starts_with("Error:") {
					return;
				}
				
				let parts: Vec<&str> = self.current_input.split_whitespace().collect();
				if parts.len() == 1 {
					// Single number
					if self.current_input.starts_with('-') && self.current_input != "-" {
						self.current_input = self.current_input[1..].to_string();
					} else if self.current_input != "0" && !self.current_input.is_empty() {
						self.current_input = format!("-{}", self.current_input);
					}
				}
			}
			Message::ToggleHistory => {
				self.show_history = !self.show_history;
			}
			Message::ClearHistory => {
				self.history.clear();
				self.history_index = 0;
			}
			Message::NavigateHistoryPrevious => {
				if !self.history.is_empty() && self.history_index < self.history.len() {
					self.history_index += 1;
					let entry_idx = self.history.len() - self.history_index;
					self.current_input = self.history[entry_idx].input.clone();
				}
			}
			Message::NavigateHistoryNext => {
				if self.history_index > 0 {
					self.history_index -= 1;
					if self.history_index == 0 {
						self.current_input = self.previous_result.clone().unwrap_or("0".to_string());
					} else {
						let entry_idx = self.history.len() - self.history_index;
						self.current_input = self.history[entry_idx].input.clone();
					}
				}
			}
		}
	}

	fn view(&self) -> Element<Message> {
		let display_text = &self.current_input;

		let display = text_input("0", display_text)
			.on_input(Message::InputChanged)
			.size(18)
			.padding(5)
			.width(Length::Fill);

		let calculator_buttons = self.create_button_grid();

		let calculator_panel = container(column![display, calculator_buttons].spacing(25))
			.padding(25)
			.width(Length::Fixed(320.0));

		if self.show_history {
			let history_panel = self.create_history_panel();
			container(
				row![
					calculator_panel,
					history_panel
				].spacing(10)
			).into()
		} else {
			calculator_panel.into()
		}
	}

	fn theme(&self) -> Theme {
		Theme::Dark
	}
}

impl Calculator {
	fn add_to_history(&mut self, input: String, output: String, is_error: bool) {
		self.history.push(HistoryEntry {
			input,
			output,
			is_error,
		});
		
		// Keep only last 50 calculations
		if self.history.len() > 50 {
			self.history.remove(0);
		}
	}
	
	fn create_button_grid(&self) -> Element<Message> {
		let spacing = 6;
		column![
			// First row: Clear functions, history, and division
			row![
				self.create_button("C", Message::Clear),
				self.create_button("H", Message::ToggleHistory),
				self.create_button("←", Message::Backspace),
				self.create_button("÷", Message::AddOperator('/')),
			].spacing(spacing).align_items(Alignment::Center),
			// Second row: 7,8,9, multiply
			row![
				self.create_button("7", Message::AddDigit('7')),
				self.create_button("8", Message::AddDigit('8')),
				self.create_button("9", Message::AddDigit('9')),
				self.create_button("×", Message::AddOperator('*')),
			].spacing(spacing).align_items(Alignment::Center),
			// Third row: 4,5,6, subtract
			row![
				self.create_button("4", Message::AddDigit('4')),
				self.create_button("5", Message::AddDigit('5')),
				self.create_button("6", Message::AddDigit('6')),
				self.create_button("−", Message::AddOperator('-')),
			].spacing(spacing).align_items(Alignment::Center),
			// Fourth row: 1,2,3, add
			row![
				self.create_button("1", Message::AddDigit('1')),
				self.create_button("2", Message::AddDigit('2')),
				self.create_button("3", Message::AddDigit('3')),
				self.create_button("+", Message::AddOperator('+')),
			].spacing(spacing).align_items(Alignment::Center),
			// Fifth row: 0, decimal, power, equals
			row![
				self.create_button("0", Message::AddDigit('0')),
				self.create_button(".", Message::AddDecimal),
				self.create_button("^", Message::AddOperator('^')),
				self.create_button("=", Message::Calculate),
			].spacing(spacing).align_items(Alignment::Center),
			// Sixth row: sign, backspace, scientific notation
			row![
				self.create_button("±", Message::ToggleSign),
				self.create_button("e", Message::AddScientificE),
				self.create_button("<", Message::NavigateHistoryNext),
				self.create_button(">", Message::NavigateHistoryPrevious),
			].spacing(spacing).align_items(Alignment::Center),
		].spacing(spacing).into()
	}
	
	fn create_button<'a>(&self, label: &str, message: Message) -> Element<'a, Message> {		
		let btn = button(text(label)
				.horizontal_alignment(Horizontal::Center)
				.vertical_alignment(Vertical::Center))
			.on_press(message.clone())
			.on_press_maybe(Some(message))
			.width(50)
			.height(35)
			.style(iced::theme::Button::Primary);

		btn.into()
	}
	
	fn create_history_panel(&self) -> Element<Message> {
		let mut history_items = column![];
		
		// Add header with navigation info
		let nav_info = if self.history.is_empty() {
			"No history".to_string()
		} else if self.history_index == 0 {
			"Current".to_string()
		} else {
			format!("{}/{}", self.history_index, self.history.len())
		};
		
		history_items = history_items.push(
			row![
				column![
					text("History").size(16),
					text(&nav_info).size(10).style(iced::theme::Text::Color(iced::Color::from_rgb(0.7, 0.7, 0.7)))
				],
				Space::with_width(Length::Fill),
				button("Clear").on_press(Message::ClearHistory)
			].align_items(Alignment::Center)
		);
		
		// Add history entries (most recent first)
		for (idx, entry) in self.history.iter().rev().enumerate() {
			let is_current = idx == self.history_index.saturating_sub(1) && self.history_index > 0;
			
			let input_text = text(&entry.input)
				.size(12)
				.style(if is_current {
					iced::theme::Text::Color(iced::Color::from_rgb(1.0, 1.0, 0.4))
				} else {
					iced::theme::Text::Color(iced::Color::WHITE)
				});
				
			let output_text = if entry.is_error {
				text(format!("Error: {}", entry.output))
					.size(12)
					.style(iced::theme::Text::Color(iced::Color::from_rgb(1.0, 0.4, 0.4)))
			} else {
				text(format!("= {}", entry.output))
					.size(12)
					.style(iced::theme::Text::Color(iced::Color::from_rgb(0.4, 1.0, 0.4)))
			};
			
			history_items = history_items.push(
				container(
					column![
						input_text,
						output_text,
						Space::with_height(5)
					]
				)
				.padding(5)
				.style(if is_current {
					iced::theme::Container::Box
				} else {
					iced::theme::Container::Transparent
				})
			);
		}
		
		if self.history.is_empty() {
			history_items = history_items.push(
				text("No calculations yet")
					.size(14)
					.style(iced::theme::Text::Color(iced::Color::from_rgb(0.7, 0.7, 0.7)))
			);
		}
		
		container(
			scrollable(history_items)
				.height(Length::Fill)
		)
		.padding(15)
		.width(Length::Fixed(300.0))
		.height(Length::Fill)
		.style(iced::theme::Container::Box)
		.into()
	}
}

fn evaluate_expression(expr: &str) -> Result<String, String> {
	let tokens: Vec<&str> = expr.split_whitespace().collect();
	
	if tokens.is_empty() {
		return Ok("0".to_string());
	}
	
	// Handle single number case
	if tokens.len() == 1 {
		return parse_number(tokens[0]).map(|bd| bd.to_string());
	}

	let mut numbers: Vec<BigNumber> = Vec::new();
	let mut operators: Vec<char> = Vec::new();

	for token in tokens {
		if let Ok(num) = parse_number(token) {
			numbers.push(num);
		} else if let Some(op) = token.chars().next() {
			if "+-*/^".contains(op) && token.len() == 1 {
				while let Some(&last_op) = operators.last() {
					if precedence(last_op) >= precedence(op) {
						apply_operation(&mut numbers, last_op)?;
						operators.pop();
					} else {
						break;
					}
				}
				operators.push(op);
			} else {
				return Err(format!("Invalid operator: {}", token));
			}
		}
	}

	while let Some(op) = operators.pop() {
		apply_operation(&mut numbers, op)?;
	}

	if numbers.len() != 1 {
		return Err("Invalid expression".to_string());
	}

	Ok(numbers.pop().unwrap().to_string_with_limit(25))
}

fn precedence(op: char) -> u8 {
	match op {
		'+' | '-' => 1,
		'*' | '/' => 2,
		'^' => 3,
		_ => 0,
	}
}

fn apply_operation(numbers: &mut Vec<BigNumber>, op: char) -> Result<(), String> {
	if numbers.len() < 2 {
		return Err("Not enough operands".to_string());
	}

	let b = numbers.pop().unwrap();
	let a = numbers.pop().unwrap();

	let result = match op {
		'+' => Ok(a.add(&b)),
		'-' => Ok(a.subtract(&b)),
		'*' => Ok(a.multiply(&b)),
		'/' => a.divide(&b, 15), // 15 decimal places precision
		'^' => a.power(&b),
		_ => Err(format!("Unknown operator: {}", op)),
	}?;

	numbers.push(result);
	Ok(())
}

fn main() -> iced::Result {
	Calculator::run(Settings {
		window: iced::window::Settings {
			size: Size::new(320.0, 450.0), // Increased height for new button row
			resizable: true,
			min_size: Some(Size::new(320.0, 450.0)),
			..Default::default()
		},
		..Default::default()
	})
}
