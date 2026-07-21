# src/background.py

from os import listdir
from os.path import isdir
from pygame import *
from src.config import *
from src.utils import fit_img_to_rect, scale_img_to_rect, write_centered_text

# create the forward and backward buttons
forward_rect = Rect(875, 560, 195, 110)
backward_rect = Rect(875, 680, 195, 110)

# title centre is (575, 50)
title_rect = Rect(375, 10, 400, 80)

# load the title image if the user has supplied one; otherwise leave it blank
title_img = None
try:
    _unscaled_title = image.load("assets/title.png")
    _scale = min(title_rect.width / _unscaled_title.get_width(), title_rect.height / _unscaled_title.get_height())
    title_img = transform.scale(_unscaled_title, (int(_unscaled_title.get_width() * _scale), int(_unscaled_title.get_height() * _scale)))
except Exception:
    title_img = None

def draw_title(screen):
    # draw the title image (if present) to the centre of the title rect
    if title_img is not None:
        screen.blit(title_img, (title_rect.centerx - title_img.get_width() // 2, title_rect.centery - title_img.get_height() // 2))
    else:
        # fall back to a text title so the header is never empty
        write_centered_text(screen, "Paint Project", title_rect.inflate(-10, -10), OUTLINE_COLOUR)

def initialize_background(screen):
    # load all the background images the user has placed in assets/backgrounds
    background_list = []
    if isdir("assets/backgrounds"):
        background_list = [image.load(f"assets/backgrounds/{img}") for img in listdir("assets/backgrounds") if img.lower().endswith((".png", ".jpg", ".jpeg", ".bmp"))]

    if background_list:
        scale_img_to_rect(screen, background_list[0], screen.get_rect())
    else:
        # no backgrounds supplied: paint a solid backdrop so the app still runs
        screen.fill((30, 144, 255))

    draw_title(screen)

    return background_list

def background_manager(screen, canvas_rect, screen_cap, background_list, button_data):
    mx, my = button_data["mx"], button_data["my"]
    mouse_up = button_data["mouse_up"]

    # if the user has not supplied any backgrounds, just keep showing the solid backdrop
    if not background_list:
        draw_background_buttons(screen, background_list, button_data)
        return background_list

    # check if the forward or backward buttons are clicked
    if forward_rect.collidepoint(mx, my) and mouse_up:
        background_list.append(background_list.pop(0))
        scale_img_to_rect(screen, background_list[0], screen.get_rect())
        screen.blit(screen_cap, canvas_rect)
        draw_title(screen)

    if backward_rect.collidepoint(mx, my) and mouse_up:
        background_list.insert(0, background_list.pop())
        scale_img_to_rect(screen, background_list[0], screen.get_rect())
        screen.blit(screen_cap, canvas_rect)
        draw_title(screen)

    draw_background_buttons(screen, background_list, button_data)

    return background_list

def draw_background_buttons(screen, background_list, button_data):
    mx, my = button_data["mx"], button_data["my"]

    # make a background for the forward and backward buttons
    draw.rect(screen, BG_COLOUR, forward_rect)
    draw.rect(screen, BG_COLOUR, backward_rect)

    if len(background_list) >= 2:
        # scale the forward and backward images to fit the buttons while maintaining aspect ratio
        fit_img_to_rect(screen, background_list[1], forward_rect)
        fit_img_to_rect(screen, background_list[-1], backward_rect)
    else:
        # not enough backgrounds to show previews; show a placeholder label instead
        write_centered_text(screen, "Add PNGs to\nassets/backgrounds/", forward_rect.inflate(-10, -10), OUTLINE_COLOUR)
        write_centered_text(screen, "to enable\nbackgrounds", backward_rect.inflate(-10, -10), OUTLINE_COLOUR)

    # draw the border around the forward and backward buttons
    draw.rect(screen, UNSELECED_COLOUR, forward_rect, 2)
    draw.rect(screen, UNSELECED_COLOUR, backward_rect, 2)

    # draw outlines around the forward and backward buttons when hovered
    if forward_rect.collidepoint(mx, my):
        draw.rect(screen, HOVER_COLOUR, forward_rect, 2)
    if backward_rect.collidepoint(mx, my):
        draw.rect(screen, HOVER_COLOUR, backward_rect, 2)