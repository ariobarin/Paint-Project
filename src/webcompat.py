# src/webcompat.py

# Helpers that let the paint project run on the desktop and in the browser (pygbag)
import sys
from pygame import image

IS_WEB = sys.platform == "emscripten"

if IS_WEB:
    import platform
else:
    from tkinter import filedialog

SAVE_PATH = "/tmp/painting.png"

# download a file from the browser's virtual file system
DOWNLOAD_JS = """(() => {
    const blob = new Blob([python.FS.readFile("%s")], {type: "image/png"});
    const link = document.createElement("a");
    link.href = URL.createObjectURL(blob);
    link.download = "painting.png";
    document.body.appendChild(link);
    link.click();
    link.remove();
    setTimeout(() => URL.revokeObjectURL(link.href), 1000);
})()"""

# open a hidden file input; pygbag copies the picked file to /tmp and sends an "upload" event
PICK_JS = """(() => {
    let picker = document.getElementById("paint_picker");
    if (!picker) {
        picker = document.createElement("input");
        picker.type = "file";
        picker.id = "paint_picker";
        picker.accept = "image/png,image/jpeg";
        picker.hidden = true;
        picker.onchange = () => transfer_uploads(picker.files, false);
        document.body.appendChild(picker);
    }
    picker.value = "";
    picker.click();
})()"""

uploads = [] # paths of images the user picked in the browser, waiting to be loaded
listening = False

def on_upload(event):
    uploads.append(event.text)

def save_image(surface):
    # save the image with a save dialog on the desktop, or as a download in the browser
    if IS_WEB:
        image.save(surface, SAVE_PATH)
        platform.window.eval(DOWNLOAD_JS % SAVE_PATH)
        return
    file_name = filedialog.asksaveasfilename(defaultextension=".png", filetypes=[("PNG files", "*.png")])
    if file_name: # if the user selected a file, save it
        image.save(surface, file_name)

def pick_image():
    # returns the picked file on the desktop; in the browser the file arrives later (see picked_image)
    global listening
    if not IS_WEB:
        return filedialog.askopenfilename(filetypes=[('Pictures','*.png; *.jpg; *.jpeg')])
    if not listening:
        platform.EventTarget.addEventListener(platform.window, "upload", on_upload)
        listening = True
    platform.window.eval(PICK_JS)
    return None

def picked_image():
    # the next image the user uploaded in the browser, if one has arrived
    return uploads.pop(0) if uploads else None
