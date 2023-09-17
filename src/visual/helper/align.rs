/// Calculate the horizontal position to center a child element within a parent element.
///
/// This function calculates the horizontal position (`x` coordinate) to center a child element
/// within a parent element, given the widths of both the parent and child elements.
///
/// # Arguments
///
/// * `parent_width` - The width of the parent element.
/// * `child_width` - The width of the child element.
///
/// # Returns
///
/// * `i64` - The horizontal position (`x` coordinate) to center the child element.
///
/// # Example
///
/// ```rust
/// let parent_width = 800;
/// let child_width = 200;
/// let center_x_position = center_x(parent_width, child_width);
/// println!("Center X Position: {}", center_x_position);
/// ```
pub fn center_x(parent_width: i64, child_width: i64) -> i64 {
    (parent_width - child_width) / 2
}

/// Calculate the vertical position to center a child element within a parent element.
///
/// This function calculates the vertical position (`y` coordinate) to center a child element
/// within a parent element, given the heights of both the parent and child elements.
///
/// # Arguments
///
/// * `parent_height` - The height of the parent element.
/// * `child_height` - The height of the child element.
///
/// # Returns
///
/// * `i64` - The vertical position (`y` coordinate) to center the child element.
///
/// # Example
///
/// ```rust
/// 
///
///
/// let parent_height = 600;
/// let child_height = 150;
///
/// let center_y_position = center_y(parent_height, child_height);
/// println!("Center Y Position: {}", center_y_position);
/// 
/// ```
pub fn center_y(parent_height: i64, child_height: i64) -> i64 {
    (parent_height - child_height) / 2
}
