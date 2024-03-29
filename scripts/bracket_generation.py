import base64
import os
import sys
import cv2
from PIL import Image, ImageFont, ImageDraw
import numpy as np

def generate_bracket_image(region, total_rounds, args):

    current_dir =  os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    region_background_mapping = {
        'Europe': 'bracket_preset_eu.png',
        'North America & South America': 'bracket_preset_nasa.png',
        'Asia & Oceania': 'bracket_preset_apac.png',
    }
    # root = os.path.dirname(os.path.dirname(current_dir))
    background_image_path = os.path.join(current_dir, "assets/brackets", region_background_mapping.get(region, 'bracket_preset_default.png'))
    font_path = os.path.join(current_dir, "assets/fonts","LilitaOne-Regular.ttf")
    background_image = cv2.imread(background_image_path)

    region = region
    
    total_rounds = int(total_rounds)
    sep = "/se/pa/ra/tor/"
    results = []
    for arg in args.split(","):

        round, match_id, player1_name, player2_name, is_winner1, is_winner2 = arg.split(sep)
        results.append((int(round), int(match_id), player1_name, player2_name, bool(is_winner1 == "true"), bool(is_winner2 == "true")))
        
    image_width = 2000
    image_height = 1000
    horizontal_padding = 80
    reference_rounds = 6
    reference_ratio = 13
    game_box_width_height_ratio = (total_rounds / reference_rounds) * reference_ratio
    
    _size = total_rounds
    _columns = _size + 1
    _column_width = image_width / _columns
    _game_box_width = _column_width - horizontal_padding
    _game_box_height = _game_box_width / game_box_width_height_ratio

    resized_background_image = cv2.resize(background_image, (image_width, image_height))

    image = resized_background_image.copy()

    total_previous_games = 0

    for i in range(_columns):
        games = 2 ** abs(i - _size)
        x_center = _column_width * (i + 0.5)
        y_size = image_height / games
        font_face = cv2.FONT_HERSHEY_DUPLEX
        reference_font_scale = 0.5
        font_scale = (reference_rounds / total_rounds) * reference_font_scale
        font_thickness = 1
        for j in range(games):
            y_center = y_size * (j + 0.5)
            cv2.rectangle(image, (int(x_center - _game_box_width / 2), int(y_center - _game_box_height / 2)), (int(x_center + _game_box_width / 2), int(y_center + _game_box_height / 2)), (192, 192, 192), -1)
    
        for j in range(games):
            y_center = y_size * (j + 0.5)

            if i != _columns - 1:
                cv2.line(image, (int(x_center + _game_box_width / 2), int(y_center)), (int(x_center + _game_box_width / 2 + horizontal_padding / 2), int(y_center)), (0, 0, 0), 1)

            if i != 0:
                cv2.line(image, (int(x_center - _game_box_width / 2), int(y_center)), (int(x_center - _game_box_width / 2 - horizontal_padding / 2), int(y_center)), (0, 0, 0), 1)

            if j % 2 == 1:
                cv2.line(image, (int(x_center + _game_box_width / 2 + horizontal_padding / 2), int(y_center)), (int(x_center + _game_box_width / 2 + horizontal_padding / 2), int(y_center - y_size)), (0, 0, 0), 1)
                
            if i == _columns - 1:
                index_winner1 = next((i for i, (_, _, _, _, is_winner1, _) in enumerate(results) if is_winner1), None)

                index_winner2 = next((i for i, (_, _, _, _, _, is_winner2) in enumerate(results) if is_winner2), None)
                
                if index_winner1 is not None:
                    _, match_id, player1_name, _, _, _ = results[index_winner1]
                    
                    text1 = f"{player1_name}"

                    text_size1, _ = cv2.getTextSize(text1, font_face, font_scale, font_thickness)

                    text_x1 = int(x_center - text_size1[0] / 2)
                    y_center1 = y_size * (((match_id * 2) - 2) + 0.5)
                    text_y1 = int(y_center1)

                    cv2.putText(image, text1, (text_x1, text_y1), font_face, font_scale, (0, 0, 0), font_thickness, cv2.LINE_AA)
                    continue

                elif index_winner2 is not None:
                    _, match_id, _, player2_name, _, _ = results[index_winner2]

                    text2 = f"{player2_name}"

                    text_size1, _ = cv2.getTextSize(text1, font_face, font_scale, font_thickness)

                    text_x1 = int(x_center - text_size1[0] / 2)
                    y_center2 = y_size * (((match_id * 2) - 2) + 0.5)
                    text_y2 = int(y_center2)

                    cv2.putText(image, text2, (text_x1, text_y2), font_face, font_scale, (0, 0, 0), font_thickness, cv2.LINE_AA)
                    continue
    
            if (total_previous_games + j) < len(results):
                round, match_id, player1_name, player2_name, is_winner1, is_winner2 = results[int(total_previous_games + j)]
    
                if round - 1 == i:
                    text1 = f"{player1_name}"
                    text2 = f"{player2_name}"

                    text_size1, _ = cv2.getTextSize(text1, font_face, font_scale, font_thickness)

                    text_x1 = int(x_center - text_size1[0] / 2)
                    y_center1 = y_size * (((match_id * 2) - 2) + 0.5)
                    text_y1 = int(y_center1)

                    y_center2 = y_size * (((match_id * 2) - 1) + 0.5)
                    text_y2 = int(y_center2)

                    image = write_text(image, text1, text_x1, text_y1, font_path, font_scale, (0, 0, 0))
                    image = write_text(image, text2, text_x1, text_y2, font_path, font_scale, (0, 0, 0))

        total_previous_games += games / 2
        
    _, buffer = cv2.imencode(".png", image)
    image_bytes = buffer.tobytes()
    image_base64 = base64.b64encode(image_bytes).decode("utf-8")
    print(image_base64)

def write_text(image, text: str, x: float, y: float, font_path: str, font_size, color):
    pil_image = Image.fromarray(cv2.cvtColor(image, cv2.COLOR_BGR2RGB))
    draw = ImageDraw.Draw(pil_image)
    position = (x,y)
    font = ImageFont.truetype(font_path, font_size)
    draw.text(position, text, font=font, fill=color)
    return cv2.cvtColor(np.array(pil_image), cv2.COLOR_RGB2BGR)

generate_bracket_image(sys.argv[1], sys.argv[2], sys.argv[3])