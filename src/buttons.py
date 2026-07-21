# src/buttons.py

from tkinter import filedialog
from pygame import *
from src.config import *
from src.utils import fit_img_to_rect, write_centered_text


init()

buttons = {
    "fill": {
        "rect": Rect(10,455,90,90)
    },
    
    "save": {
        "rect": Rect(1080, 560, 110, 110)
    },
    "load": {
        "rect": Rect(1080, 680, 110, 110)
    },

    "trash": {
        "rect": Rect(110, 10, 40, 40)
    }
}


def draw_fill_button(screen, button_data, fill):
    mx, my = button_data["mx"], button_data["my"]

    # draw the fill button
    if fill:
        draw.rect(screen, SELECTED_COLOUR, buttons["fill"]["rect"])
    else:
        draw.rect(screen, BG_COLOUR, buttons["fill"]["rect"])

    # draw the outline of the fill button
    if buttons["fill"]["rect"].collidepoint(mx, my):
        draw.rect(screen, HOVER_COLOUR, buttons["fill"]["rect"], 2)
    else:
        draw.rect(screen, UNSELECED_COLOUR, buttons["fill"]["rect"], 2)

    # write the text "Fill" in the centre of the button
    write_centered_text(screen, "Fill", buttons["fill"]["rect"], (0,0,0))

# load all the button images
button_imgs = {}
for button in buttons:
    if button == "fill":
        continue
    button_imgs[button] = image.load(f"assets/buttons/{button}.png")

def draw_buttons(screen, button_data, fill):
    mx, my = button_data["mx"], button_data["my"]
    for button in buttons:
        # draw background of button
        draw.rect(screen, BG_COLOUR, buttons[button]["rect"])

        if button == "fill":
            continue
        
        # draw the button image
        img = button_imgs[button]
        img_rect = buttons[button]["rect"]
        fit_img_to_rect(screen, img, img_rect.inflate(-15, -15))
        rect = buttons[button]["rect"]

        # draw the border around the button
        if rect.collidepoint(mx, my):
            draw.rect(screen, HOVER_COLOUR, rect, 2)
        else:
            draw.rect(screen, UNSELECED_COLOUR, rect, 2)

    # draw the fill button (it's a special case because it's not an image)
    draw_fill_button(screen,button_data,  fill)

def button_manager(screen, canvas_rect, button_data, tool_settings):
    mouse_up = button_data["mouse_up"]
    fill = tool_settings["fill"]
    key_x = button_data["key_x"]

    draw_buttons(screen, button_data, fill)

    pushed_button = None
    updated = False
    if mouse_up:
        for button in buttons: # check if a button is clicked
            if buttons[button]["rect"].collidepoint(button_data["mx"], button_data["my"]):
                pushed_button = button
                break
        match pushed_button: # check which button was clicked
            case "fill":
                fill = not fill
            case "save":
                save(screen, canvas_rect)
            case "load":
                updated = load(screen, canvas_rect)
            case "trash":
                trash(screen, canvas_rect)
                updated = True
    if key_x: # check if the user pressed the x key
        trash(screen, canvas_rect)
        updated = True
    return fill, updated

def save(screen, canvas_rect):
    # save the canvas to a file (give the user a dialogue of where they want to save the file) .png
    file_name = filedialog.asksaveasfilename(defaultextension=".png", filetypes=[("PNG files", "*.png")])
    if file_name: # if the user selected a file, save it
        image.save(screen.subsurface(canvas_rect), file_name)

def load(screen, canvas_rect):
    # load a file to the canvas (give the user a dialogue of where the file is) .png
    file_name = filedialog.askopenfilename(filetypes=[('Pictures','*.png; *.jpg; *.jpeg')])

    # if the user selected a file, load it
    if file_name:
        # clear the canvas
        draw.rect(screen, (255,255,255), canvas_rect)

        # load the image
        img = image.load(file_name)
        # check if the image is smaller than the canvas
        if img.get_width() < canvas_rect.width and img.get_height() < canvas_rect.height:
            # if it is, blit the image to the canvas
            screen.blit(img, canvas_rect.topleft)
        else:
            # if the image is larger than the canvas, scale the image to fit the canvas while maintaining aspect ratio
            scale = min(canvas_rect.width / img.get_width(), canvas_rect.height / img.get_height())
            scaled_img = transform.scale(img, (int(img.get_width() * scale), int(img.get_height() * scale)))
            screen.blit(scaled_img, canvas_rect.topleft)

    return True if file_name else False

def trash(screen, canvas_rect):
    # clear the canvas
    draw.rect(screen, (255, 255, 255), canvas_rect)

    return True