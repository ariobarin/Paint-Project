# src/utils.py

from math import hypot
from pygame import *
import pygame.font as pygame_font

pygame_font.init()

def gradient_rect(window, left_colour, right_colour, rect):
    # Create a 2x2 gradient surface and scale it to the size of the rect
    colour_rect = Surface((2, 2))
    draw.line(colour_rect, left_colour, (0, 0), (0, 1))
    draw.line(colour_rect, right_colour, (1, 0), (1, 1))
    colour_rect = transform.smoothscale(colour_rect, (rect.width, rect.height))
    window.blit(colour_rect, rect)
    
def write_centered_text(screen, text, rect, colour, cache={}):
    rect_tuple = (rect.x, rect.y, rect.width, rect.height) # Convert rect to a tuple
    key = (text, rect_tuple, colour) # Use the tuple as the key
    if key in cache: # If the text has already been rendered, use the cached version
        for text_surface, x, y in cache[key]:
            screen.blit(text_surface, (x, y))
        return

    # Split the text into lines and calculate the size of the text
    lines = text.split("\n")
    font_size = 1
    font_obj = pygame_font.Font(None, font_size)
    line_sizes = [(line, font_obj.size(line)) for line in lines]
    widest_line, widest_size = max(line_sizes, key=lambda item: item[1][0])
    text_height = sum(size[1] for _, size in line_sizes)
    text_width = widest_size[0]

    # binary search for the maximum font size
    low, high = font_size, max(rect.width, rect.height)
    while low < high:
        mid = (low + high + 1) // 2
        font_obj = pygame_font.Font(None, mid)
        line_sizes = [(line, font_obj.size(line)) for line in lines]
        text_height = sum(size[1] for _, size in line_sizes)
        text_width = max(size[0] for _, size in line_sizes)

        if text_height <= rect.height and text_width <= rect.width:
            low = mid
        else:
            high = mid - 1

    # Render the text with the maximum font size
    font_size = low
    font_obj = pygame_font.Font(None, font_size)
    line_sizes = [(line, font_obj.size(line)) for line in lines]
    text_height = sum(size[1] for _, size in line_sizes)
    text_width = max(size[0] for _, size in line_sizes)

    cache[key] = []
    y = rect.y + (rect.height - text_height) // 2
    for i, (line, size) in enumerate(line_sizes): # Render each line of text
        text_surface = font_obj.render(line, True, colour)
        x = rect.x + (rect.width - size[0]) // 2
        screen.blit(text_surface, (x, y + i * size[1]))

        # Cache the rendered text
        cache[key].append((text_surface, x, y + i * size[1]))

def draw_dotted_rect(screen, color, rect, dash_length):
    x, y, w, h = rect
    # Draw the horizontal dashed lines
    for i in range(0, w, dash_length * 2):
        draw.line(screen, color, (x + i, y), (x + i + dash_length, y), 3)
        draw.line(screen, color, (x + i, y + h), (x + i + dash_length, y + h), 3)

    # Draw the vertical dashed lines
    for i in range(0, h, dash_length * 2):
        draw.line(screen, color, (x, y + i), (x, y + i + dash_length), 3)
        draw.line(screen, color, (x + w, y + i), (x + w, y + i + dash_length), 3)

def draw_rounded_line(screen, col, start, end, radius):
    dx, dy = end[0] - start[0], end[1] - start[1]
    dist = hypot(dx, dy)
    if dist == 0: # Prevent division by zero
        return
    dx, dy = dx / dist, dy / dist
    for i in range(int(dist)): # Draw a circle at each point along the line
        x = int(start[0] + dx * i)
        y = int(start[1] + dy * i)
        draw.circle(screen, col, (x, y), radius)

def fit_img_to_rect(screen, img, rect, cache={}):
    key = (img, (rect.x, rect.y, rect.width, rect.height))  # Convert rect to tuple
    if key in cache: # If the image has already been scaled, use the cached version
        scaled_img = cache[key]
    else:
        scale = min(rect.width / img.get_width(), rect.height / img.get_height())
        img_size = (int(img.get_width() * scale), int(img.get_height() * scale))
        scaled_img = transform.smoothscale(img, img_size)
        # Cache the scaled image
        cache[key] = scaled_img

    screen.blit(scaled_img, (rect.centerx - scaled_img.get_width() // 2, rect.centery - scaled_img.get_height() // 2))

def scale_img_to_rect(screen, img, rect, cache={}):
    key = (img, (rect.x, rect.y, rect.width, rect.height))  # Convert rect to tuple
    if key in cache: # If the image has already been scaled, use the cached version
        scaled_img = cache[key]
    else:
        scale = max(rect.width / img.get_width(), rect.height / img.get_height())
        img_size = (int(img.get_width() * scale), int(img.get_height() * scale))
        scaled_img = transform.smoothscale(img, img_size)
        # Cache the scaled image
        cache[key] = scaled_img

    screen.blit(scaled_img, (rect.centerx - scaled_img.get_width() // 2, rect.centery - scaled_img.get_height() // 2))