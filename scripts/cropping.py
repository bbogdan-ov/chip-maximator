import os
import math
from PIL import Image

import util

# Use alpha threshold because Blender can produse some noise
ALPHA_THRESHOLD = 0.002

def image_bounding_box(layer_name, img):
	pixels = img.load()

	min_x, min_y = img.width + 1, img.height + 1
	max_x, max_y = 0, 0

	for y in range(img.height):
		for x in range(img.width):
			pixel = pixels[(x, y)]
			is_transparent = pixel[3] <= ALPHA_THRESHOLD

			if not is_transparent:
				min_x = min(min_x, x)
				min_y = min(min_y, y)

				max_x = max(max_x, x)
				max_y = max(max_y, y)

	assert min_x < max_x, f"Layer '{layer_name}' (min_x = {min_x}, max_x = {max_x})"
	assert min_y < max_y, f"Layer '{layer_name}' (min_y = {min_y}, max_y = {max_y})"

	return {
		"x": min_x,
		"y": min_y,
		"width": max_x - min_x + 1,
		"height": max_y - min_y + 1
	}


images = []
boundings = []


def crop_layer_frames(
	layer_name,
	crop_x,
	crop_y,
	crop_width,
	crop_height,
	crop_align_x,
	crop_align_y,
):
	dir_path = util.get_layer_out_dir(layer_name)

	# Calculate bounding box of all images in the dir
	for file in sorted(os.listdir(dir_path)):
		filepath = f"{dir_path}/{file}"
		if not os.path.isfile(filepath):
			continue

		image = Image.open(filepath)
		bounding = image_bounding_box(layer_name, image)

		images.append((filepath, image))
		boundings.append(bounding)

	# Calculate largest bounding box size
	largest_size = (0, 0)
	for bounding in boundings:
		largest_size = (
			max(largest_size[0], bounding["width"]),
			max(largest_size[1], bounding["height"])
		)

	if crop_width > 0:
		largest_size = (crop_width, largest_size[1])
	if crop_height > 0:
		largest_size = (largest_size[0], crop_height)

	# Crop every image to the largest size
	for index, (filepath, image) in enumerate(images):
		bounding = boundings[index]

		width = largest_size[0]
		height = largest_size[1]
		x = None
		y = None

		if crop_align_x <= -1:
			# Left
			x = bounding["x"]
		elif crop_align_x == 0:
			# Center
			x = bounding["x"] - math.ceil((width - bounding["width"]) / 2)
		elif crop_align_x >= 1:
			# Top
			x = bounding["x"] - (width - bounding["width"])

		if crop_align_y <= -1:
			# Top
			y = bounding["y"]
		elif crop_align_y == 0:
			# Center
			y = bounding["y"] - math.ceil((height - bounding["height"]) / 2)
		elif crop_align_y >= 1:
			# Bottom
			y = bounding["y"] - (height - bounding["height"])

		if crop_x >= 0 and crop_y >= 0:
			x = crop_x
			y = crop_y

		crop = (x, y, x + width, y + height)

		image.crop(crop).save(filepath)
		image.close()

	images[:] = []
	boundings[:] = []

	return largest_size
