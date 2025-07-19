import bpy
import os

import util


def setup_render_settings():
	image = bpy.context.scene.render.image_settings
	scene = bpy.context.scene

	scene.render.resolution_x = 700
	scene.render.resolution_y = 700
	scene.render.fps = 8

	scene.render.engine = "BLENDER_EEVEE_NEXT"
	scene.eevee.taa_render_samples = 64
	scene.eevee.use_raytracing = False

	image.file_format = "PNG"
	image.color_mode = "RGBA"


def toggle_all_tracks(name):
	for obj in bpy.context.scene.objects:
		# Show every object in render
		obj.hide_render = False
		obj.visible_camera = True

		if obj.animation_data == None:
			continue

		for track in obj.animation_data.nla_tracks:
			track.mute = not (track.name == name or track.name == "*")


def setup_layer_composition(layer):
	scene = bpy.context.scene
	node_tree = scene.node_tree

	for node in node_tree.nodes:
		# Do nothing if we find Render Layers node with label matching layer's name
		if node.type == "R_LAYERS" and node.label == layer.name:
			return False;

	# Create Render Layers node
	render_node = node_tree.nodes.new("CompositorNodeRLayers")
	render_node.label = layer.name
	render_node.scene = scene
	render_node.layer = layer.name
	
	# Create Custom Group node
	group_node = node_tree.nodes.new("CompositorNodeGroup")
	group_node.label = layer.name
	group_node.node_tree = bpy.data.node_groups.get("Default Composite")
	group_node.mute = layer.my_settings.is_uv

	# Create Composite node
	compose_node = node_tree.nodes.new("CompositorNodeComposite")
	compose_node.label = layer.name
	compose_node.use_alpha = True

	# Link nodes together
	node_tree.links.new(group_node.inputs[0], render_node.outputs[0])
	node_tree.links.new(compose_node.inputs[0], group_node.outputs[0])

	return True

def setup_layer(layer):
	scene = bpy.context.scene
	image = scene.render.image_settings
	settings = layer.my_settings

	# Select layer's composite node
	for node in scene.node_tree.nodes:
		node.mute = settings.is_uv and node.type == "GROUP" and node.label == layer.name

		if node.type == "COMPOSITE" and node.label == layer.name:
			scene.node_tree.nodes.active = node
			break
	else:
		return f"No composite node found for layer '{layer.name}'"

	if settings.is_uv:
		scene.render.use_motion_blur = False
		image.compression = 0
		image.color_depth = "16"
		scene.view_settings.view_transform = "Raw"
	else:
		scene.render.use_motion_blur = settings.motion_blur
		image.compression = 10
		image.color_depth = "8"
		scene.view_settings.view_transform = "Standard"

	# Reset current animation and mute animation tracks != layer name
	scene.frame_current = 0
	toggle_all_tracks(layer.name)

	# Set frame range
	scene.frame_current = settings.frame_current
	scene.frame_start = settings.frame_start
	scene.frame_end = settings.frame_end

	# Set output path
	scene.render.filepath = util.get_layer_out_dir(layer.name)

	return None
