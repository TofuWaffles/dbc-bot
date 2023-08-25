
/// A trait for stripping quotes from a string.
pub trait QuoteStripper {
  /// Strip double quotes from the string and return a new String.
  fn strip_quote(&self) -> String;
}

impl QuoteStripper for String {
  /// Strip double quotes from the string and return a new String.
  ///
  /// # Examples
  ///
  /// ```
  /// let s = String::from("\"Hello, world!\"");
  /// let stripped = s.strip_quote();
  /// assert_eq!(stripped, "Hello, world!");
  /// ```
  fn strip_quote(&self) -> String {
      let mut result = String::new();

      for c in self.chars() {
          if c != '"' {
              result.push(c);
          }
      }

      result
  }
}