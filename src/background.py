# src/background.py

from os import listdir
from pygame import *
from src.config import *
from src.utils import fit_img_to_rect, scale_img_to_rect

# create the forward and backward buttons
forward_rect = Rect(875, 560, 195, 110)
backward_rect = Rect(875, 680, 195, 110)

# title centre is (575, 50)
title_rect = Rect(375, 10, 400, 80)
unscaled_title = image.load("assets/title.png")

# scale the title image to fit the title rect while maintaining aspect ratio
scale = min(title_rect.width / unscaled_title.get_width(), title_rect.height / unscaled_title.get_height())
title_img = transform.scale(unscaled_title, (int(unscaled_title.get_width() * scale), int(unscaled_title.get_height() * scale)))

def draw_title(screen):
    # draw the title image to the centre of the title rect
    screen.blit(title_img, (title_rect.centerx - title_img.get_width() // 2, title_rect.centery - title_img.get_height() // 2))

def initialize_background(screen):
    # load all the background images
    background_list = [image.load(f"assets/backgrounds/{img}") for img in sorted(listdir("assets/backgrounds"))]
    screen.blit(background_list[0], (0, 0))

    draw_title(screen)

    return background_list

def background_manager(screen, canvas_rect, screen_cap, background_list, button_data):
    mx, my = button_data["mx"], button_data["my"]
    mouse_up = button_data["mouse_up"]

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

    # scale the forward and backward images to fit the buttons while maintaining aspect ratio
    fit_img_to_rect(screen, background_list[1], forward_rect)
    fit_img_to_rect(screen, background_list[-1], backward_rect)

    # draw the border around the forward and backward buttons
    draw.rect(screen, UNSELECED_COLOUR, forward_rect, 2)
    draw.rect(screen, UNSELECED_COLOUR, backward_rect, 2)
    
    # draw outlines around the forward and backward buttons when hovered
    if forward_rect.collidepoint(mx, my):
        draw.rect(screen, HOVER_COLOUR, forward_rect, 2)
    if backward_rect.collidepoint(mx, my):
        draw.rect(screen, HOVER_COLOUR, backward_rect, 2)