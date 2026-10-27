# src/music_player.py

from os import listdir, path
import re
from pygame import *
from src.config import *
from src.utils import fit_img_to_rect, write_centered_text

mixer.init()

# create the music buttons
music_buttons = {
    "previous_track": {
        "rect": Rect(1000, 55, 40, 40)
    },
    "play": {
        "rect": Rect(1050, 55, 40, 40)
    },
    "pause": {
        "rect": Rect(1100, 55, 40, 40)
    },
    "next_track": {
        "rect": Rect(1150, 55, 40, 40)
    }
}

# create a rect to display the currently playing song
currently_playing_rect = Rect(800, 10, 190, 40)

# add the music button images to the music_buttons dictionary
for button_name in music_buttons:
    # load the image
    button_img = image.load(f"assets/music_buttons/{button_name}.png")
    # scale the image to the button size
    button_img = transform.scale(button_img, music_buttons[button_name]["rect"].size)
    # add the image to the dictionary
    music_buttons[button_name]["image"] = button_img

# create a variable to store if the music is paused
paused = False

def initialize_music():
    # the browser build leaves the music out, so there may be no music folder
    if not path.isdir("assets/music"):
        return []

    # get the music tracks from the assets/music folder (end with .mp3)
    music_tracks = ["assets/music/" + music for music in listdir("assets/music") if music.endswith(".mp3")]
    mixer.music.load(music_tracks[0])
    mixer.music.play()
    return music_tracks

def music_player(screen, button_data, tool_settings, music_tracks):
    mx, my = button_data["mx"], button_data["my"]
    mouse_up = button_data["mouse_up"]
    volume = tool_settings["volume"] / 100
    global paused

    if not music_tracks:
        return music_tracks

    # set the volume of the music
    mixer.music.set_volume(volume)

    # draw the music buttons
    draw_music_buttons(screen, button_data)

    # check if the user has clicked on the music buttons
    if mouse_up and music_buttons["pause"]["rect"].collidepoint(mx, my) and mixer.music.get_busy():
        paused = True
        mixer.music.pause()
        return music_tracks

    if mouse_up:
        for button_name in music_buttons:
            button = music_buttons[button_name]
            if button["rect"].collidepoint(mx, my):
                if button_name == "previous_track": # 1, 2, 3, 4 -> 4, 1, 2, 3
                    music_tracks.insert(0, music_tracks.pop())
                    mixer.music.load(music_tracks[0])
                    mixer.music.play()
                elif button_name == "play": # play the music if it is paused
                    if mixer.music.get_busy():
                        continue
                    else:
                        paused = False
                        mixer.music.unpause()
                elif button_name == "next_track": # 1, 2, 3, 4 -> 2, 3, 4, 1
                    music_tracks.append(music_tracks.pop(0))
                    mixer.music.load(music_tracks[0])
                    mixer.music.play()

    # check if the current song has ended
    if not mixer.music.get_busy() and not paused: # 1, 2, 3, 4 -> 2, 3, 4, 1
        # similar to next_track
        music_tracks.append(music_tracks.pop(0))
        mixer.music.load(music_tracks[0])
        mixer.music.play()
        

    # draw a background for the current track
    draw.rect(screen, BG_COLOUR, currently_playing_rect)

    # draw a border for the background
    draw.rect(screen, OUTLINE_COLOUR, currently_playing_rect, 2)

    # get the name of the current song
    song_name = music_tracks[0].split("/")[-1] # 02. Ripped Pants.mp3 | 04. F.U.N. Song.mp3
    
    # remove the track number and .mp3 from the song name
    song_name = re.findall(r"\d+\.\s(.+)\.mp3", song_name)[0]    

    # create a rect to display the song name
    text_rect = currently_playing_rect.inflate(-5, -5)

    # display currently playing
    write_centered_text(screen, song_name, text_rect, SELECTED_COLOUR)

    return music_tracks

def draw_music_buttons(screen, button_data):
    mx, my = button_data["mx"], button_data["my"]
    
    for button_name in music_buttons:
        button = music_buttons[button_name]
        
        # draw a background for the button
        draw.rect(screen, BG_COLOUR, button["rect"])

        # draw the button image
        fit_img_to_rect(screen, button["image"], button["rect"].inflate(-10, -10))

        # draw the border around the button
        if button["rect"].collidepoint(mx, my):
            draw.rect(screen, HOVER_COLOUR, button["rect"], 2)
        else:
            draw.rect(screen, UNSELECED_COLOUR, button["rect"], 2)