import os
import io
import base64
import json
import sys
from PIL import Image, ImageDraw, ImageFont

def generate_text_image(text, font_size, font_color, stroke_width, stroke_color):
    current_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    font_path = os.path.join(current_dir, "assets/fonts", "LilitaOne-Regular.ttf")
    font = ImageFont.truetype(font_path, font_size)
    r,g,b,a = hex_to_rgba(font_color)
    sr,sg,sb,sa = hex_to_rgba(stroke_color)
    # Calculate image size
    (width, height) = get_text_dimensions(text, font)
    
    # Create image
    canvas = Image.new("RGBA", (width, height), (0, 0, 0, 0))
    draw = ImageDraw.Draw(canvas)
    
    # Draw text
    draw.text((5, 0), text=text, fill=(r,g,b), font=font, stroke_width=stroke_width, stroke_fill=(sr, sg, sb))
    
    # Save image to buffer
    buffered = io.BytesIO()
    canvas.save(buffered, format="PNG")
    
    # Encode image to base64
    img_str = base64.b64encode(buffered.getvalue()).decode("utf-8")
    print(img_str)

def get_text_dimensions(text_string, font):
    _ , descent = font.getmetrics()

    text_width = font.getmask(text_string).getbbox()[2]
    text_height = font.getmask(text_string).getbbox()[3] + descent

    return (text_width+10, text_height+5)

def hex_to_rgba(hex_color):
    hex_color = hex_color.lstrip('#')  # Remove '#' if present
    hex_color = int(hex_color, 16)     # Convert hex to integer

    # Extract RGBA components using bitwise operations
    red = hex_color >> 24
    green = hex_color >> 16
    blue = hex_color >> 8
    alpha = (hex_color << 24) >> 24
    return red, green, blue, alpha

if __name__ == "__main__":
    args = json.loads(sys.argv[1])
    generate_text_image(
        text=args["text"],
        font_size=int(args["font_size"]),
        font_color=args["font_color"],
        stroke_width=int(args["stroke_width"]),
        stroke_color=args["stroke_color"]
    )