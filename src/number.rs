


use num_bigint::BigInt;
use num_traits::{Zero, One};
use regex::Regex;

// Custom BigDecimal implementation for high precision arithmetic
#[derive(Debug, Clone, PartialEq)]
pub struct BigNumber {
	mantissa: BigInt,
	scale: i32, // Number of decimal places
}

impl BigNumber {
	fn new(mantissa: BigInt, scale: i32) -> Self {
		Self { mantissa, scale }
	}
	

	fn from_str(s: &str) -> Result<Self, String> {
		let s = s.trim();
		
		if s.is_empty() {
			return Err("Empty string".to_string());
		}
		
		// Check for scientific notation (case insensitive, with optional whitespace)
		if s.contains(|c| c == 'e' || c == 'E') {
			return Self::from_scientific(s);
		}
		
		Self::from_decimal(s)
	}

	fn from_decimal(s: &str) -> Result<Self, String> {
		let s = s.trim();
		
		// Handle sign prefix
		let (sign, num_str) = match s.chars().next() {
			Some('+') => (1, &s[1..]),
			Some('-') => (-1, &s[1..]),
			_ => (1, s),
		};
		
		if num_str.is_empty() {
			return Err("Missing digits after sign".to_string());
		}
		
		if let Some(dot_pos) = num_str.find('.') {
			// Handle multiple decimal points
			if num_str[dot_pos + 1..].contains('.') {
				return Err("Multiple decimal points".to_string());
			}
			
			let scale = (num_str.len() - dot_pos - 1) as i32;
			let mantissa_str = num_str.replace('.', "");
			
			// Handle cases like ".123" or "123."
			if mantissa_str.is_empty() {
				return Err("Missing digits around decimal point".to_string());
			}
			
			let mantissa = mantissa_str.parse::<BigInt>()
				.map_err(|e| format!("Invalid decimal format: {}", e))?;
			
			Ok(Self::new(mantissa * sign, scale))
		} else {
			let mantissa = num_str.parse::<BigInt>()
				.map_err(|e| format!("Invalid integer format: {}", e))?;
			
			Ok(Self::new(mantissa * sign, 0))
		}
	}

	fn from_scientific(s: &str) -> Result<Self, String> {
		let s = s.trim();
		
		// More flexible regex that allows whitespace around components
		let re = Regex::new(r"(?ix)
			^\s*                          # Optional leading whitespace
			([+-]?\s*\d*\.?\d+)           # Base (with optional sign, decimal, digits)
			\s*[eE]\s*                    # Exponent separator with optional whitespace
			([+-]?\s*\d+)                 # Exponent (with optional sign, digits)
			\s*$                          # Optional trailing whitespace
		").unwrap();
		
		if let Some(caps) = re.captures(s) {
			let base_str = caps[1].replace(char::is_whitespace, "");
			let exp_str = caps[2].replace(char::is_whitespace, "");
			
			let exp: i32 = exp_str.parse()
				.map_err(|e| format!("Invalid exponent '{}': {}", exp_str, e))?;
			
			let base = Self::from_decimal(&base_str)?;
			Ok(base.multiply_by_power_of_10(exp))
		} else {
			Err(format!("Invalid scientific notation: '{}'", s))
		}
	}
	
	fn multiply_by_power_of_10(&self, exp: i32) -> Self {
		Self::new(self.mantissa.clone(), self.scale - exp)
	}
	
	fn normalize(&self) -> Self {
		if self.mantissa.is_zero() {
			return Self::new(BigInt::zero(), 0);
		}
		
		let mut mantissa = self.mantissa.clone();
		let mut scale = self.scale;
		
		// Remove trailing zeros
		while scale > 0 && &mantissa % 10 == BigInt::zero() {
			mantissa /= 10;
			scale -= 1;
		}
		
		Self::new(mantissa, scale)
	}
	
	fn align_scales(&self, other: &Self) -> (BigNumber, BigNumber) {
		let max_scale = self.scale.max(other.scale);
		let left = self.scale_to(max_scale);
		let right = other.scale_to(max_scale);
		(left, right)
	}
	
	fn scale_to(&self, target_scale: i32) -> Self {
		if self.scale == target_scale {
			return self.clone();
		}
		
		let scale_diff = target_scale - self.scale;
		if scale_diff > 0 {
			let factor = BigInt::from(10).pow(scale_diff as u32);
			Self::new(&self.mantissa * factor, target_scale)
		} else {
			let factor = BigInt::from(10).pow((-scale_diff) as u32);
			Self::new(&self.mantissa / factor, target_scale)
		}
	}
	
	pub fn add(&self, other: &Self) -> Self {
		let (left, right) = self.align_scales(other);
		Self::new(&left.mantissa + &right.mantissa, left.scale).normalize()
	}
	
	pub fn subtract(&self, other: &Self) -> Self {
		let (left, right) = self.align_scales(other);
		Self::new(&left.mantissa - &right.mantissa, left.scale).normalize()
	}
	
	pub fn multiply(&self, other: &Self) -> Self {
		let mantissa = &self.mantissa * &other.mantissa;
		let scale = self.scale + other.scale;
		Self::new(mantissa, scale).normalize()
	}
	
	pub fn divide(&self, other: &Self, precision: i32) -> Result<Self, String> {
		if other.mantissa.is_zero() {
			return Err("Division by zero".to_string());
		}
		
		// Scale up the dividend to achieve desired precision
		let scale_up = precision + other.scale - self.scale;
		let dividend = if scale_up > 0 {
			&self.mantissa * BigInt::from(10).pow(scale_up as u32)
		} else {
			self.mantissa.clone()
		};
		
		let quotient = dividend / &other.mantissa;
		let result_scale = if scale_up > 0 { 
			scale_up 
		} else { 
			self.scale - other.scale 
		};
		
		Ok(Self::new(quotient, result_scale).normalize())
	}
	
	pub fn power(&self, exponent: &Self) -> Result<Self, String> {
		// Simple integer power implementation
		if exponent.scale > 0 {
			return Err("Non-integer exponents not supported".to_string());
		}
		
		let exp_int = exponent.mantissa.to_string().parse::<i32>()
			.map_err(|_| "Exponent too large")?;
		
		if exp_int < 0 {
			return Err("Negative exponents not supported".to_string());
		}
		
		if exp_int == 0 {
			return Ok(Self::new(BigInt::one(), 0));
		}
		
		let mut result = self.clone();
		for _ in 1..exp_int {
			result = result.multiply(self);
		}
		
		Ok(result.normalize())
	}
	
	pub fn to_string(&self) -> String {
		self.to_string_with_limit(25) // Default limit for display
	}
	
	pub fn to_string_with_limit(&self, max_chars: usize) -> String {
		let standard_form = self.to_standard_string();
		
		if standard_form.len() <= max_chars {
			return standard_form;
		}
		
		// Convert to scientific notation if too long
		self.to_scientific_notation()
	}
	
	fn to_standard_string(&self) -> String {
		if self.scale <= 0 {
			let zeros = "0".repeat((-self.scale) as usize);
			return format!("{}{}", self.mantissa, zeros);
		}
		
		let mantissa_str = self.mantissa.to_string();
		let is_negative = mantissa_str.starts_with('-');
		let abs_str = if is_negative { &mantissa_str[1..] } else { &mantissa_str };
		
		if self.scale >= abs_str.len() as i32 {
			let leading_zeros = "0".repeat((self.scale as usize) - abs_str.len());
			let result = format!("0.{}{}", leading_zeros, abs_str);
			if is_negative { format!("-{}", result) } else { result }
		} else {
			let split_pos = abs_str.len() - (self.scale as usize);
			let integer_part = &abs_str[..split_pos];
			let decimal_part = &abs_str[split_pos..];
			let result = format!("{}.{}", integer_part, decimal_part);
			if is_negative { format!("-{}", result) } else { result }
		}
	}
	
	fn to_scientific_notation(&self) -> String {
		if self.mantissa.is_zero() {
			return "0".to_string();
		}
		
		let mantissa_str = self.mantissa.to_string();
		let is_negative = mantissa_str.starts_with('-');
		let abs_str = if is_negative { &mantissa_str[1..] } else { &mantissa_str };
		
		if abs_str.is_empty() {
			return "0".to_string();
		}
		
		// Find the position of the most significant digit
		let significant_digits: Vec<char> = abs_str.chars().collect();
		
		// Calculate the exponent
		let exponent = (significant_digits.len() as i32) - 1 - self.scale;
		
		// Format the mantissa (keep first digit, then decimal point, then up to 10 more digits)
		let mut formatted_mantissa = String::new();
		if is_negative {
			formatted_mantissa.push('-');
		}
		
		formatted_mantissa.push(significant_digits[0]);
		
		if significant_digits.len() > 1 {
			formatted_mantissa.push('.');
			// Take up to 10 digits after the decimal point for scientific notation
			let remaining_digits: String = significant_digits[1..].iter()
				.take(10)
				.collect();
			// Remove trailing zeros in scientific notation
			let trimmed = remaining_digits.trim_end_matches('0');
			if !trimmed.is_empty() {
				formatted_mantissa.push_str(trimmed);
			}
		}
		
		format!("{}e{}", formatted_mantissa, exponent)
	}
}



pub fn parse_number(s: &str) -> Result<BigNumber, String> {
	BigNumber::from_str(s)
}
