import asyncio
from pygame import *
from src.background import background_manager, initialize_background
from src.buttons import button_manager
from src.music_player import initialize_music, music_player
from src.sliders import *
from src.stamp import initialize_stamps, stamp_manager
from src.tools import draw_tools, tool_manager, tool_change
from src.undo_redo import undo_manager

'''
Paint Project - Ario Barin Ostovary
Features:
    Tools:
        Pencil
        Eraser
        Spray
        Paint Brush
        Line (hold shift to snap to 90 degrees) - Thickness
        Rectangle (hold shift for square) - Fill/No Fill
        Ellipse - (hold shift for circle) - Fill/No Fill
        Eye Dropper
        Fireworks
        Airbrush (beautiful)
        Paint Bucket (fill region with color)
        Fill Screen (fill screen with color)
        Stamp (with size slider while the stamp tool is selected) (15 stamps)
        --> Select tool (select a region of the canvas and move it around) (grab the resize handles to resize the selection)

    Tool Descriptions (when hovering over the tool buttons)

    Undo/Redo (ctrl+z, ctrl+y, or click the undo/redo buttons)

    Save/Load

    Clear Canvas (trash can button or x)

    Color selection with sliders (displays hex code of the color)

    Changing background (8 different backgrounds)

    Full music player (play, pause, next, previous, volume slider, current song display)

    Changing size with slider (for tools)
'''

async def main():
    width,height=1200,800
    screen=display.set_mode((width,height))

    # load all the background images
    background_list = initialize_background(screen)
    # load all the stamp images
    stamp_list = initialize_stamps()
    # load all the music tracks
    music_tracks = initialize_music()

    canvas_rect=Rect(110,100,1080,450)
    draw.rect(screen,(255,255,255),canvas_rect)

    myClock=time.Clock()

    redraw_tools = ["line", "rect", "ellipse", "stamp", "select"]
    omx,omy = 0,0
    dmx,dmy = 0,0
    tool="pencil"
    col=(99,179,212)
    radius = 10
    stamp_size = 10
    volume = 2
    fill = False
    tool_settings = {"tool": tool, "col": col, "radius": radius, "fill": fill, "stamp": stamp_list[0], "stamp_size": stamp_size, "volume": volume}

    undo_list = [screen.subsurface(canvas_rect).copy()]
    redo_list = []

    running=True
    while running:
        done_move = updated = dont_update = mouse_up = mouse_down = key_x = key_y = key_z = False # reset variables

        for evt in event.get():
            if evt.type==QUIT:
                running=False
            if evt.type==MOUSEBUTTONDOWN:
                mouse_down = True
                dmx,dmy = mouse.get_pos()
            if evt.type==MOUSEBUTTONUP:
                mouse_up = True
            if evt.type==KEYDOWN:
                if evt.key==K_x:
                    key_x = True
                if evt.key==K_y:
                    key_y = True
                if evt.key==K_z:
                    key_z = True

        # check if the user is holding down the shift key or the ctrl key
        keys = key.get_pressed()
        shift = True if keys[K_LSHIFT] or keys[K_RSHIFT] else False
        ctrl = True if keys[K_LCTRL] or keys[K_RCTRL] else False

        mx,my=mouse.get_pos()
        mb=mouse.get_pressed()

        # create a dictionary of all the data that needs to be passed to the functions
        button_data = {
            "mb": mb,
            "mx": mx,
            "my": my,
            "omx": omx,
            "omy": omy,
            "dmx": dmx,
            "dmy": dmy,
            "mouse_up": mouse_up,
            "mouse_down": mouse_down,

            "shift": shift,
            "ctrl": ctrl,

            "key_x": key_x,
            "key_y": key_y,
            "key_z": key_z
            }

        # draw the background, stamps, music player, and buttons
        background_list = background_manager(screen, canvas_rect, undo_list[-1], background_list, button_data)
        stamp_list = stamp_manager(screen, stamp_list, button_data)
        music_tracks = music_player(screen, button_data, tool_settings, music_tracks)

        # check for tool changes
        tool = tool_change(tool_settings, button_data)

        # draw the colour palette and radius selector
        col = draw_colour_palette(screen, button_data, tool_settings)
        radius = draw_radius_selector(screen, button_data, tool_settings)

        # draw the buttons and check if they are clicked
        fill, updated = button_manager(screen, canvas_rect, button_data, tool_settings)

        # undo and redo
        undo_list, redo_list = undo_manager(screen, canvas_rect, undo_list, redo_list, button_data)

        # draw the tools and check if they are clicked
        draw_tools(screen, tool_settings, button_data)

        if updated: # if the canvas has been updated, add the new canvas to the undo list
            undo_list.append(screen.subsurface(canvas_rect).copy())
            redo_list = []

        if tool in redraw_tools: # if the tool requires the canvas to be redrawn, redraw the canvas
            screen.blit(undo_list[-1], canvas_rect)

        # draw the stamp sizer and volume slider and update the tool settings
        stamp_size, dont_update = draw_stamp_sizer(screen, canvas_rect, button_data, tool_settings, stamp_list)
        if music_tracks: # no music folder (like in the browser build) means no music player
            volume = draw_volume_slider(screen, button_data, tool_settings)
        tool_settings = {"tool": tool, "col": col, "radius": radius, "fill": fill, "stamp": stamp_list[0], "stamp_size": stamp_size, "volume": volume}
        
        # check if the user is using the tool
        if canvas_rect.collidepoint(dmx,dmy) or tool_settings["tool"] == "select":
            screen.set_clip(canvas_rect)
            tool_settings["col"], done_move = tool_manager(screen, canvas_rect, button_data, tool_settings)
            screen.set_clip(None)

        # add the new canvas to the undo list if the user has finished using the tool
        if (mouse_up and canvas_rect.collidepoint(dmx,dmy) and tool != "select" or done_move) and not dont_update:
            undo_list.append(screen.subsurface(canvas_rect).copy())
            redo_list = []

        # draw border around canvas
        draw.rect(screen, (0, 0, 0), canvas_rect, 5)
        omx,omy=mx,my
        myClock.tick(60)  # Adjust the frame rate as needed
        display.flip()
        display.set_caption("Spongebob Paint Project - Ario Barin Ostovary")
        await asyncio.sleep(0) # let the browser draw the frame (does nothing on the desktop)
        

    quit()

if __name__ == "__main__":
    asyncio.run(main())