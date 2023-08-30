import base64
import os
import sys
import cv2

def generate_bracket_image(results_arg):

    current_dir = os.path.dirname(os.path.abspath(__file__))

    background_image_path = os.path.join(current_dir, "presets", "preset_bracket.jpg")
    
    background_image = cv2.imread(background_image_path)

    results = []
    for result in results_arg.split(","):
        player1_name, player2_name, result = result.split("|")
        results.append((player1_name, player2_name, result))

    image_width = 2000
    image_height = 1000
    horizontal_padding = 70
    game_box_width_height_ratio = 3
    
    _size = 4
    _columns = _size * 2 + 1
    _column_width = image_width / _columns
    _game_box_width = _column_width - horizontal_padding
    _game_box_height = _game_box_width / game_box_width_height_ratio

    resized_background_image = cv2.resize(background_image, (image_width, image_height))

    image = resized_background_image.copy()

    for i in range(_columns):
        if i - _size < 0:
            side = "LEFT"
        elif i - _size > 0:
            side = "RIGHT"
        else:
            side = "CENTER"
        games = 2 ** abs(i - _size)
        x_center = _column_width * (i + 0.5)
        y_size = image_height / games
        font_face = cv2.FONT_HERSHEY_COMPLEX_SMALL
        font_scale = 0.2
        font_thickness = 1
        for j in range(games):
            y_center = y_size * (j + 0.5)
            cv2.rectangle(image, (int(x_center - _game_box_width / 2), int(y_center - _game_box_height / 2)), (int(x_center + _game_box_width / 2), int(y_center + _game_box_height / 2)), (192, 192, 192), -1)
            if j < len(results):
                player1_name, player2_name, result = results[j]
                text = f"{player1_name} vs {player2_name}"
                text_size, _ = cv2.getTextSize(text, font_face, font_scale, font_thickness)
                text_x = int(x_center - text_size[0] / 2)
                text_y = int(y_center + text_size[1] / 2)
            cv2.putText(image, text, (text_x, text_y), font_face, font_scale, (0, 0, 0), font_thickness, cv2.LINE_AA)
            if i != _columns - 1:
                cv2.line(image, (int(x_center + _game_box_width / 2), int(y_center)), (int(x_center + _game_box_width / 2 + horizontal_padding / 2), int(y_center)), (0, 0, 0), 1)
            if i != 0:
                cv2.line(image, (int(x_center - _game_box_width / 2), int(y_center)), (int(x_center - _game_box_width / 2 - horizontal_padding / 2), int(y_center)), (0, 0, 0), 1)

            if j % 2 == 1:
                if side == "LEFT":
                    cv2.line(image, (int(x_center + _game_box_width / 2 + horizontal_padding / 2), int(y_center)), (int(x_center + _game_box_width / 2 + horizontal_padding / 2), int(y_center - y_size)), (0, 0, 0), 1)
                if side == "RIGHT":
                    cv2.line(image, (int(x_center - _game_box_width / 2 - horizontal_padding / 2), int(y_center)), (int(x_center - _game_box_width / 2 - horizontal_padding / 2), int(y_center - y_size)), (0, 0, 0), 1)

    _, buffer = cv2.imencode(".png", image)
    image_bytes = buffer.tobytes()
    image_base64 = base64.b64encode(image_bytes).decode("utf-8")
    print(image_base64)

generate_bracket_image(sys.argv[1])