use poise::serenity_prelude::Colour;

/// Converts a string result into a corresponding color represented as a `poise::serenity_prelude::Colour` struct.
///
/// This function takes a `result` parameter, which is a string indicating the result of an event.
/// It matches the input string to predefined cases and returns a `poise::serenity_prelude::Colour` value representing the
/// associated color.
///
/// # Arguments
///
/// * `result` - A string representing the result of an event ("victory", "defeat", "draw").
///
/// # Returns
///
/// A `poise::serenity_prelude::Colour` struct representing the color associated with the input result. If the input result
/// is not recognized (i.e., not "victory", "defeat", or "draw"), the function returns a default
/// color (black).
///
/// # Examples
///
/// ```
/// use your_crate_name::color;
/// use poise::serenity_prelude::Colour;
///
/// let victory_color = color("victory".to_string());
/// assert_eq!(victory_color, Colour::new(0x00800)); // Green
///
/// let defeat_color = color("defeat".to_string());
/// assert_eq!(defeat_color, Colour::new(0xFF0000)); // Red
///
/// let draw_color = color("draw".to_string());
/// assert_eq!(draw_color, Colour::new(0xFFFFFF)); // White
///
/// let unknown_color = color("unknown".to_string());
/// assert_eq!(unknown_color, Colour::new(0x000000)); // Default (black)
/// ```
pub fn get_color(result: String) -> Colour {
    match result.as_str() {
        "victory" => Colour::new(u32::from_str_radix("90EE90", 16).unwrap()), // Green
        "defeat" => Colour::new(u32::from_str_radix("FF0000", 16).unwrap()),  // Red
        "draw" => Colour::new(u32::from_str_radix("FFFFFF", 16).unwrap()),    // White
        _ => Colour::new(000000), // Default color (black) for unknown cases
    }
}
