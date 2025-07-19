import os
import math
from PIL import Image

import util


def _make_spritesheet_image(layer_name, stack=False):
	frame_width = None
	frame_height = None
	frames = []

	dir_path = util.get_layer_out_dir(layer_name)

	for file in sorted(os.listdir(dir_path)):
		filepath = f"{dir_path}/{file}"
		if not os.path.isfile(filepath):
			continue

		image = Image.open(filepath)

		if frame_width == None: frame_width = image.width
		if frame_height == None: frame_height = image.height

		assert frame_width == image.width, f"All images in spritesheet must have the same width (expected: {frame_width}, got: {image.width})"
		assert frame_height == image.height, f"All images in spritesheet must have the same height (expected: {frame_height}, got: {image.height})"

		frames.append(image)

	if len(frames) == 1:
		return frames[0]

	width = frame_width * len(frames)
	height = frame_height

	if stack:
		width = int(width / 2)
		height *= 2

	spritesheet = Image.new("RGBA", (width, height))
	for index, frame in enumerate(frames):
		if stack:
			offset_x = math.floor(index / 2) * frame_width
			offset_y = (index % 2) * frame_height
			spritesheet.paste(frame, (offset_x, offset_y))
		else:
			offset_x = index * frame_width
			spritesheet.paste(frame, (offset_x, 0))

		frame.close()

	return spritesheet

def make_spritesheet(layer_name, stack=False, is_uv=False):
	image = _make_spritesheet_image(layer_name, stack)
	save_to = util.get_layer_result_path(layer_name)

	if is_uv:
		alpha = image.getchannel(3)

		bg = Image.new("RGBA", image.size, (0, 0, 255, 255))
		bg.paste(image, mask=alpha)

		bg.save(save_to, compress_level=0)
		bg.close()
		image.close()
	else:
		image.save(save_to)
		image.close()
