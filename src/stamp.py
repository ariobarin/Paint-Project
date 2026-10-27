# src/stamp.py

from os import listdir
from pygame import *
from src.config import *
from src.utils import fit_img_to_rect

previous_preview = Rect(585, 690, 100, 100)
next_preview = Rect(690, 690, 100, 100)

def initialize_stamps():
    # load all the images in the assets/stamps folder w/o scaling them
    stamp_list = [image.load(f"assets/stamps/{img}") for img in sorted(listdir("assets/stamps"))]
    return stamp_list

def stamp_manager(screen, stamp_list, button_data):
    mx, my = button_data["mx"], button_data["my"]
    mouse_up = button_data["mouse_up"]

    # check if the user has clicked on the next or previous buttons
    if next_preview.collidepoint(mx, my) and mouse_up:
        stamp_list.append(stamp_list.pop(0))
    if previous_preview.collidepoint(mx, my) and mouse_up:
        stamp_list.insert(0, stamp_list.pop())

    draw_stamp_buttons(screen, stamp_list, button_data)
    return stamp_list

def draw_stamp_buttons(screen, stamp_list, button_data):
    mx, my = button_data["mx"], button_data["my"]

    # draw a background behind the buttons
    draw.rect(screen, (0,0,0), (previous_preview.x, previous_preview.y, previous_preview.w, previous_preview.h))
    draw.rect(screen, (0,0,0), (next_preview.x, next_preview.y, next_preview.w, next_preview.h))
    
    # draw the images of the next and previous stamps
    fit_img_to_rect(screen, stamp_list[-1], previous_preview)
    fit_img_to_rect(screen, stamp_list[1], next_preview)

    # draw the outline of the buttons
    draw.rect(screen, UNSELECED_COLOUR, next_preview, 2)
    draw.rect(screen, UNSELECED_COLOUR, previous_preview, 2)
    
    # draw the hover effect
    if next_preview.collidepoint(mx, my):
        draw.rect(screen, HOVER_COLOUR, next_preview, 2)
    if previous_preview.collidepoint(mx, my):
        draw.rect(screen, HOVER_COLOUR, previous_preview, 2)