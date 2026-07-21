# src/sliders.py

from pygame import *
from src.utils import *
from src.config import *

# create the slider rects
red_slider = Rect(10, 675, 400, 35)
green_slider = Rect(10, 715, 400, 35)
blue_slider = Rect(10, 755, 400, 35)
colour_rect = Rect(420, 560, 155, 230)

radius_slider = Rect(10, 635, 400, 35)

stamp_slider = Rect(795,560,70,230)

volume_slider = Rect(1000, 10, 190, 40)


def draw_colour_palette(screen, button_data, tool_settings): # horizontal sliders, each slider is a gradient from black to a colour, the position of the slider determines the colour
    col = tool_settings["col"]
    mb = button_data["mb"]
    mx, my = button_data["mx"], button_data["my"]

    # draw the gradient rectangles
    gradient_rect(screen, (255,0,0), (0,0,0), red_slider)
    gradient_rect(screen, (0,255,0), (0,0,0), green_slider)
    gradient_rect(screen, (0,0,255), (0,0,0), blue_slider)

    # draw outlines of the sliders
    draw.rect(screen, UNSELECED_COLOUR, red_slider, 2)
    draw.rect(screen, UNSELECED_COLOUR, green_slider, 2)
    draw.rect(screen, UNSELECED_COLOUR, blue_slider, 2)

    r, g, b = col
    # check if the user is clicking on the sliders
    if mb[0]:
        # set the colour based on the position of the sliders
        if red_slider.collidepoint(mx, my):
            r = (red_slider.x + red_slider.w - mx) * 255 // red_slider.w
        if green_slider.collidepoint(mx, my):
            g = (green_slider.x + green_slider.w - mx) * 255 // green_slider.w
        if blue_slider.collidepoint(mx, my):
            b = (blue_slider.x + blue_slider.w - mx) * 255 // blue_slider.w

    col = (r, g, b)
    # draw the current colour
    draw.rect(screen, col, colour_rect)

    # draw the outline of the colour rect
    draw.rect(screen, UNSELECED_COLOUR, colour_rect, 2)

    # make the text a colour that stands out against the colour rect
    txt_col = (255,255,255) if r + g + b < 384 else (0,0,0)

    # draw the hex code of the colour in the centre of the colour rect
    hex_code = "#{:02x}{:02x}{:02x}".format(r, g, b).upper()
    text_rect = colour_rect.inflate(-5, -5)
    write_centered_text(screen, hex_code, text_rect, txt_col)

    # draw sliders on top of the gradient rectangles
    r_pos = red_slider.x + red_slider.w - r * red_slider.w // 255
    g_pos = green_slider.x + green_slider.w - g * green_slider.w // 255
    b_pos = blue_slider.x + blue_slider.w - b * blue_slider.w // 255

    # use max and min to make sure the sliders are within the slider rect
    r_pos = max(red_slider.x, min(red_slider.x + red_slider.w, r_pos))
    g_pos = max(green_slider.x, min(green_slider.x + green_slider.w, g_pos))
    b_pos = max(blue_slider.x, min(blue_slider.x + blue_slider.w, b_pos))

    # draw the sliders
    draw.rect(screen, OUTLINE_COLOUR, (r_pos, red_slider.y, 2, red_slider.h))
    draw.rect(screen, OUTLINE_COLOUR, (g_pos, green_slider.y, 2, green_slider.h))
    draw.rect(screen, OUTLINE_COLOUR, (b_pos, blue_slider.y, 2, blue_slider.h))
    return col

def draw_radius_selector(screen, button_data, tool_settings):
    mx, my = button_data["mx"], button_data["my"]
    mb = button_data["mb"]
    radius = tool_settings["radius"]
    fill_col = radius * 255 // 100

    # draw the slider
    draw.rect(screen, (fill_col,fill_col,fill_col), radius_slider)
    draw.rect(screen, UNSELECED_COLOUR, radius_slider, 2)

    # check if the user is clicking on the slider
    if mb[0] and radius_slider.collidepoint(mx, my):
        radius = max(1, (mx - radius_slider.x) * 100 // radius_slider.w)  # Set a minimum limit of 1

    # calculate the position of the slider
    slider_pos = radius_slider.x + radius * radius_slider.w // 100

    # make sure the slider is within the slider rect
    slider_pos = max(radius_slider.x, min(radius_slider.x + radius_slider.w, slider_pos))

    # make the slider a bit more visible against the background and the text
    slider_col = (196,196,196) if fill_col < 128 else (64,64,64)
    draw.rect(screen, slider_col, (slider_pos, radius_slider.y, 2, radius_slider.h))

    # make the text a colour that stands out against the slider
    txt_col = (255,255,255) if fill_col < 128 else (0,0,0)

    # draw the text on top of the slider (Size)
    text_rect = radius_slider.inflate(-5, -5)
    write_centered_text(screen, "Size", text_rect, txt_col)
    return radius

def draw_stamp_sizer(screen, canvas_rect, button_data, tool_settings, stamp_list): # vertical slider, stamp_size is a percentage, 100% when slider is at the top
    dont_update = False
    mx, my = button_data["mx"], button_data["my"]
    dmx, dmy = button_data["dmx"], button_data["dmy"]
    mb = button_data["mb"]
    mouse_up = button_data["mouse_up"]
    stamp_size = tool_settings["stamp_size"]
    fill_col = stamp_size * 255 // 100

    # draw the slider
    draw.rect(screen, (fill_col,fill_col,fill_col), stamp_slider)
    draw.rect(screen, UNSELECED_COLOUR, stamp_slider, 2)

    # check if the user is clicking on the slider
    if (mb[0] or mouse_up) and stamp_slider.collidepoint(dmx, dmy):
        # 100 at the top, 0 at the bottom
        stamp_size = (stamp_slider.y + stamp_slider.h - my) * 100 // stamp_slider.h

        # 1 < stamp_size < 100
        stamp_size = max(1, min(100, stamp_size))
        if tool_settings["tool"] == "stamp" and stamp_list:
            dont_update = True
            # draw a preview of the stamp with the current size onto the canvas
            stamp = stamp_list[0]

            # scale the stamp to fit within the canvas while maintaining aspect ratio
            scale = min(canvas_rect.width / stamp.get_width(), canvas_rect.height / stamp.get_height())

            # multiply the scale by the stamp size percentage
            scale *= stamp_size / 100

            # scale the stamp
            scaled_stamp = transform.scale(stamp, (int(stamp.get_width() * scale), int(stamp.get_height() * scale)))

            # draw the stamp in the center of the canvas
            screen.blit(scaled_stamp, (canvas_rect.centerx - scaled_stamp.get_width() // 2, canvas_rect.centery - scaled_stamp.get_height() // 2))

    # calculate the position of the slider
    slider_pos = stamp_slider.y + stamp_slider.h - stamp_size * stamp_slider.h // 100
    
    # make sure the slider is within the slider rect
    slider_pos = max(stamp_slider.y, min(stamp_slider.y + stamp_slider.h, slider_pos))

    # make the slider a bit more visible against the background and the text
    slider_col = (196,196,196) if fill_col < 128 else (64,64,64)
    draw.rect(screen, slider_col, (stamp_slider.x, slider_pos, stamp_slider.w, 2))

    # make the text a colour that stands out against the slider
    txt_col = (255,255,255) if fill_col < 128 else (0,0,0)

    # draw the text on top of the slider (Stamp\nSize)
    text_rect = stamp_slider.inflate(-5, -5)
    write_centered_text(screen, "Stamp\nSize", text_rect, txt_col)

    return stamp_size, dont_update

def draw_volume_slider(screen, button_data, tool_settings): # horizontal slider, volume is a percentage, 100% when slider is at the right, 0% when slider is at the left
    mx, my = button_data["mx"], button_data["my"]
    mb = button_data["mb"]
    volume = tool_settings["volume"]

    fill_col = volume * 255 // 100 

    # draw the slider
    draw.rect(screen, (fill_col,fill_col,fill_col), volume_slider)
    draw.rect(screen, UNSELECED_COLOUR, volume_slider, 2)

    # check if the user is clicking on the slider
    if mb[0] and volume_slider.collidepoint(mx, my):
        volume = (mx - volume_slider.x) * 100 // volume_slider.w

    slider_pos = volume_slider.x + volume * volume_slider.w // 100

    # make sure the slider is within the slider rect
    slider_pos = max(volume_slider.x, min(volume_slider.x + volume_slider.w, slider_pos))

    # make the slider a bit more visible against the background and the text
    slider_col = (196,196,196) if fill_col < 128 else (64,64,64)
    draw.rect(screen, slider_col, (slider_pos, volume_slider.y, 2, volume_slider.h))

    # make the text a colour that stands out against the slider
    txt_col = (255,255,255) if fill_col < 128 else (0,0,0)

    
    text_rect = volume_slider.inflate(-5, -5)
    write_centered_text(screen, "Volume", text_rect, txt_col)

    return volume