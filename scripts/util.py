import os
import bpy

BLEND_DIR = os.path.dirname(bpy.data.filepath)
OUT_DIR = BLEND_DIR + "/out/"
TEXTURES_DIR = os.path.abspath(BLEND_DIR + "/../assets/textures/")

def to_kebab_case(s):
	words = s.split(" ")
	return "-".join(map(lambda w: w.lower(), words))

# Returns layer's path to render directory
# All layer's rendered frames will be here
def get_layer_out_dir(layer_name):
	return f"{OUT_DIR}/{ to_kebab_case(layer_name) }/"

# Returns layer's path to work-ready spritesheet/texture
def get_layer_result_path(layer_name):
	return f"{TEXTURES_DIR}/{ to_kebab_case(layer_name) }.png"

