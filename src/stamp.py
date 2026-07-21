# src/stamp.py

from os import listdir
from os.path import isdir
from pygame import *
from src.config import *
from src.utils import fit_img_to_rect, write_centered_text

previous_preview = Rect(585, 690, 100, 100)
next_preview = Rect(690, 690, 100, 100)

def initialize_stamps():
    # load all the images the user has placed in assets/stamps (any common image format)
    if not isdir("assets/stamps"):
        return []
    stamp_list = [image.load(f"assets/stamps/{img}") for img in listdir("assets/stamps") if img.lower().endswith((".png", ".jpg", ".jpeg", ".bmp"))]
    return stamp_list

def stamp_manager(screen, stamp_list, button_data):
    mx, my = button_data["mx"], button_data["my"]
    mouse_up = button_data["mouse_up"]

    # only cycle when the user has supplied at least one stamp
    if len(stamp_list) > 0:
        if next_preview.collidepoint(mx, my) and mouse_up:
            stamp_list.append(stamp_list.pop(0))
        if previous_preview.collidepoint(mx, my) and mouse_up and len(stamp_list) > 1:
            stamp_list.insert(0, stamp_list.pop())

    draw_stamp_buttons(screen, stamp_list, button_data)
    return stamp_list

def draw_stamp_buttons(screen, stamp_list, button_data):
    mx, my = button_data["mx"], button_data["my"]

    # draw a background behind the buttons
    draw.rect(screen, (0,0,0), (previous_preview.x, previous_preview.y, previous_preview.w, previous_preview.h))
    draw.rect(screen, (0,0,0), (next_preview.x, next_preview.y, next_preview.w, next_preview.h))

    if len(stamp_list) >= 2:
        # draw the images of the next and previous stamps
        fit_img_to_rect(screen, stamp_list[-1], previous_preview)
        fit_img_to_rect(screen, stamp_list[1], next_preview)
    elif len(stamp_list) == 1:
        # only one stamp: show it in both previews
        fit_img_to_rect(screen, stamp_list[0], previous_preview)
        fit_img_to_rect(screen, stamp_list[0], next_preview)
    else:
        # no stamps supplied: show a placeholder telling the user where to add them
        write_centered_text(screen, "Add PNGs to\nassets/stamps/", previous_preview.inflate(-6, -6), OUTLINE_COLOUR)
        write_centered_text(screen, "to enable\nstamps", next_preview.inflate(-6, -6), OUTLINE_COLOUR)

    # draw the outline of the buttons
    draw.rect(screen, UNSELECED_COLOUR, next_preview, 2)
    draw.rect(screen, UNSELECED_COLOUR, previous_preview, 2)

    # draw the hover effect
    if next_preview.collidepoint(mx, my):
        draw.rect(screen, HOVER_COLOUR, next_preview, 2)
    if previous_preview.collidepoint(mx, my):
        draw.rect(screen, HOVER_COLOUR, previous_preview, 2)
