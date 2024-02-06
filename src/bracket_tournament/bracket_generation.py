import base64
import os
import sys
import cv2

def generate_bracket_image(region, total_rounds, args):

    current_dir = os.path.dirname(os.path.abspath(__file__))

    if region == 'EU':
        background_image_path = os.path.join(current_dir, "presets", "bracket_preset_eu.jpg")
    elif region == 'NASA':
        background_image_path = os.path.join(current_dir, "presets", "bracket_preset_nasa.jpg")
    elif region == 'APAC':
        background_image_path = os.path.join(current_dir, "presets", "bracket_preset_apac.jpg")
    
    background_image = cv2.imread(background_image_path)

    region = region
    
    total_rounds = total_rounds

    results = []
    for arg in args.split(","):
        round, match_id, player1_name, player2_name = arg.split("|")
        results.append((int(round), int(match_id), player1_name, player2_name))

    image_width = 2000
    image_height = 1000
    horizontal_padding = 80
    game_box_width_height_ratio = 10
    
    _size = int(total_rounds)
    _columns = _size + 1
    _column_width = image_width / _columns
    _game_box_width = _column_width - horizontal_padding
    _game_box_height = _game_box_width / game_box_width_height_ratio

    resized_background_image = cv2.resize(background_image, (image_width, image_height))

    image = resized_background_image.copy()

    for i in range(_columns):
        games = 2 ** abs(i - _size)
        x_center = _column_width * (i + 0.5)
        y_size = image_height / games
        font_face = cv2.FONT_HERSHEY_PLAIN
        font_scale = 0.5
        font_thickness = 1
        for j in range(games):
            y_center = y_size * (j + 0.5)
            cv2.rectangle(image, (int(x_center - _game_box_width / 2), int(y_center - _game_box_height / 2)), (int(x_center + _game_box_width / 2), int(y_center + _game_box_height / 2)), (192, 192, 192), -1)
    
        for j in range(games):
            y_center = y_size * (j + 0.5)
            if j % 2 == 1:
                y_center1 = y_size * ((j + 1) + 0.5)
            else:
                y_center1 = y_size * (j + 0.5)
    
            if j < len(results):
                round, match_id, player1_name, player2_name = results[j]
    
                # Check if the player is in the correct round
                if round - 1 == i:
                    # Calculate text position based on seed
                    text1 = f"{player1_name}"
                    text2 = f"{player2_name}"

                    # Calculate positions for player1_name and player2_name
                    text_size1, _ = cv2.getTextSize(text1, font_face, font_scale, font_thickness)
                    text_size2, _ = cv2.getTextSize(text2, font_face, font_scale, font_thickness)

                    text_x1 = int(x_center - text_size1[0] / 2)
                    text_y1 = int(y_center1)
            
                    # Calculate the position of the next rectangle for the second text
                    y_center2 = y_size * (((match_id * 2) - 1) + 0.5)
                    text_y2 = int(y_center2)

                    # Draw the text for player1_name and player2_name in separate boxes
                    cv2.putText(image, text1, (text_x1, text_y1), font_face, font_scale, (0, 0, 0), font_thickness, cv2.LINE_AA)
                    cv2.putText(image, text2, (text_x1, text_y2), font_face, font_scale, (0, 0, 0), font_thickness, cv2.LINE_AA)

            if i != _columns - 1:
                cv2.line(image, (int(x_center + _game_box_width / 2), int(y_center)), (int(x_center + _game_box_width / 2 + horizontal_padding / 2), int(y_center)), (0, 0, 0), 1)
            if i != 0:
                cv2.line(image, (int(x_center - _game_box_width / 2), int(y_center)), (int(x_center - _game_box_width / 2 - horizontal_padding / 2), int(y_center)), (0, 0, 0), 1)

            if j % 2 == 1:
                cv2.line(image, (int(x_center + _game_box_width / 2 + horizontal_padding / 2), int(y_center)), (int(x_center + _game_box_width / 2 + horizontal_padding / 2), int(y_center - y_size)), (0, 0, 0), 1)

    _, buffer = cv2.imencode(".png", image)
    image_bytes = buffer.tobytes()
    image_base64 = base64.b64encode(image_bytes).decode("utf-8")
    print(image_base64)

generate_bracket_image(sys.argv[1], sys.argv[2], sys.argv[3])