# Blender Python script to automate rendering lot of textures

import os
import sys
import bpy
import importlib
import shutil
import time

# Add scripts/ dir into path so we load local modules
blend_dir = os.path.dirname(bpy.data.filepath)
scripts_dir = os.path.abspath(blend_dir + "/../scripts")
if scripts_dir not in sys.path:
	sys.path.append(scripts_dir)

import util
importlib.reload(util)

import rendering
importlib.reload(rendering)

import spritesheet
importlib.reload(spritesheet)

import cropping
importlib.reload(cropping)

###

def is_layer_renderable(layer):
	settings = layer.my_settings
	return settings.enabled and settings.renders


def crop_layer(layer):
	settings = layer.my_settings
	crop_size = None

	# Crop layer frames
	if settings.crop and not settings.is_uv:
		crop_size = cropping.crop_layer_frames(
			layer.name,
			settings.crop_x,
			settings.crop_y,
			settings.crop_width,
			settings.crop_height,
			settings.crop_align_x,
			settings.crop_align_y,
		)

	# Generate spritesheet from layer frames
	spritesheet.make_spritesheet(
		layer.name,
		settings.stack_frames,
		settings.is_uv
	)

	return crop_size

# ====================
# Settings
# ====================

# Layer settings
class LayerSettings(bpy.types.PropertyGroup):
	enabled: bpy.props.BoolProperty(
		name = "Enabled",
		description = "Whether enable view layer for rendering",
		default = True
	)
	renders: bpy.props.BoolProperty(
		name = "Renders",
		description = "Whether the view layer will be rendered or not",
		default = True
	)

	crop: bpy.props.BoolProperty(name = "Crop", default = False)
	crop_x: bpy.props.IntProperty(name = "Crop X", default = -1, min = -1)
	crop_y: bpy.props.IntProperty(name = "Crop Y", default = -1, min = -1)
	crop_width: bpy.props.IntProperty(name = "Crop Width", default = 0, min = 0)
	crop_height: bpy.props.IntProperty(name = "Crop Height", default = 0, min = 0)
	crop_align_x: bpy.props.IntProperty(name = "Crop Align X", default = 0, min = -1, max = 1, subtype = "FACTOR")
	crop_align_y: bpy.props.IntProperty(name = "Crop Align Y", default = 0, min = -1, max = 1, subtype = "FACTOR")

	stack_frames: bpy.props.BoolProperty(name = "Stack Frames", default = False)
	motion_blur: bpy.props.BoolProperty(name = "Motion Blur", default = True)
	is_uv: bpy.props.BoolProperty(name = "Is UV", default = False)

	frame_current: bpy.props.IntProperty(name = "Frame Current", default = 0)
	frame_start: bpy.props.IntProperty(name = "Frame Start", default = 0)
	frame_end: bpy.props.IntProperty(name = "Frame End", default = 0)

	def register():
		bpy.types.ViewLayer.my_settings = bpy.props.PointerProperty(type = LayerSettings)

	def unregister():
		del bpy.types.ViewLayer.my_settings

	def draw(layout, layer, show_enabled=True):
		settings = layer.my_settings

		if show_enabled:
			layout.prop(settings, "enabled")
			row = layout.row()
			row.enabled = settings.enabled
			row.prop(settings, "renders")

		col = layout.column()
		col.use_property_split = True

		col.prop(settings, "crop")
		col.prop(settings, "crop_x")
		col.prop(settings, "crop_y", text = "Y")
		col.prop(settings, "crop_width")
		col.prop(settings, "crop_height", text = "Height")
		col.prop(settings, "crop_align_x")
		col.prop(settings, "crop_align_y", text = "Y")

		col.separator()

		col.prop(settings, "stack_frames")
		col.prop(settings, "motion_blur")
		col.prop(settings, "is_uv")

		col.separator()

		col.prop(settings, "frame_current")
		col.prop(settings, "frame_start", text = "Start")
		col.prop(settings, "frame_end", text = "End")

	def draw_panel(layout, layer):
		header, pan = layout.box().panel("layer-" + layer.name)
		header.prop(layer.my_settings, "renders", text = "")
		header.label(text = layer.name)
		if pan != None:
			LayerSettings.draw(pan, layer, False)
			pan.separator()

# Render settings
class RenderSettings(bpy.types.PropertyGroup):
	def register():
		bpy.types.Scene.my_render_settings = bpy.props.PointerProperty(type = RenderSettings)

	def unregister():
		del bpy.types.Scene.my_render_settings

	def draw(layout, scene):
		settings = scene.my_render_settings

		layout.label(text = "Layers setting")
		col = layout.column()
		for layer in scene.view_layers:
			if layer.my_settings.enabled:
				LayerSettings.draw_panel(col, layer)

# ====================
# Operators
# ====================

# Render operator
class RenderOperator(bpy.types.Operator):
	bl_idname = "my_render.render"
	bl_label = "Render"

	def execute(self, context):
		scene = context.scene
		settings = scene.my_render_settings

		self.report({"INFO"}, "=== Start rendering... ===")

		# Setup render settings
		rendering.setup_render_settings()

		start_time = time.time();

		# Render all enabled layers
		for layer in scene.view_layers:
			if not is_layer_renderable(layer):
				continue

			start_layer_time = time.time();

			# Delete layer's /out dir
			out_dir = util.get_layer_out_dir(layer.name)
			if os.path.exists(out_dir):
				shutil.rmtree(out_dir)

			rendering.setup_layer_composition(layer)
			warn = rendering.setup_layer(layer)
			if warn: self.report({"WARNING"}, warn)

			# Render frames
			bpy.ops.render.render(
				animation = True,
				write_still = True,
				layer = layer.name,
			)

			crop_size = crop_layer(layer)

			secs = time.time() - start_layer_time
			self.report({"INFO"}, f"Layer '{layer.name}' rendered and cropped to {crop_size} in {secs:.2f}s")

		secs = time.time() - start_time
		self.report({"INFO"}, f"=== Done in {secs:.2f}s! ===")
		return {"FINISHED"}

# Setup operator
class SetupRenderSettingsOperator(bpy.types.Operator):
	bl_idname = "my_render.setup_render_settings"
	bl_description = "You don't have to press this button, everything will happen automatically"
	bl_label = "Setup render settings"

	def execute(self, context):
		# Setup render settings
		rendering.setup_render_settings()

		# Setup all enabled layers
		for layer in context.scene.view_layers:
			if not is_layer_renderable(layer):
				continue

			warn = rendering.setup_layer(layer)
			if warn: self.report({"WARNING"}, warn)

		self.report({"INFO"}, "=== Layers render settings setup ===")
		return {"FINISHED"}

# Setup composition operator
class SetupCompositionOperator(bpy.types.Operator):
	bl_idname = "my_render.setup_layer_composition"
	bl_description = "Setup layer specific composition nodes\nYou don't have to press this button, everything will happen automatically"
	bl_label = "Setup composition"

	def execute(self, context):
		# Crop all enabled layers
		for layer in context.scene.view_layers:
			if not is_layer_renderable(layer):
				continue

			rendering.setup_layer_composition(layer)

		self.report({"INFO"}, "=== Layers composition setup ===")
		return {"FINISHED"}

# Crop operator
class CropOperator(bpy.types.Operator):
	bl_idname = "my_render.crop"
	bl_description = "Crop and tile enabled layers\nYou don't have to press this button, everything will happen automatically"
	bl_label = "Crop and Tile"

	def execute(self, context):
		# Crop all enabled layers
		for layer in context.scene.view_layers:
			if not is_layer_renderable(layer):
				continue

			crop_size = crop_layer(layer)
			self.report({"INFO"}, f"Layer '{layer.name}' cropped to {crop_size}")

		self.report({"INFO"}, "=== Layers cropped ===")
		return {"FINISHED"}

# ====================
# Panels
# ====================

# Layer settings panel
class LayerSettingsPanel(bpy.types.Panel):
	bl_label = "Layer Settings! Yay!"
	bl_idname = "my_render.layer_settings_panel"
	bl_space_type = "PROPERTIES"
	bl_region_type = "WINDOW"
	bl_context = "view_layer"

	def draw(self, context):
		LayerSettings.draw(self.layout, context.view_layer)


# Render panel
class RenderPanel(bpy.types.Panel):
	bl_label = "Render Buttons! Yay!"
	bl_idname = "my_render.render_panel"
	bl_space_type = "PROPERTIES"
	bl_region_type = "WINDOW"
	bl_context = "render"

	def draw(self, context):
		layout = self.layout

		RenderSettings.draw(layout, context.scene)

		layout.label(text = "Do something:")
		layout.operator("my_render.render")
		layout.operator("my_render.setup_render_settings")
		layout.operator("my_render.setup_layer_composition")
		layout.operator("my_render.crop")


register_classes = (
	RenderSettings,
	LayerSettings,

	RenderOperator,
	SetupRenderSettingsOperator,
	SetupCompositionOperator,
	CropOperator,

	RenderPanel,
	LayerSettingsPanel,
)

def register():
	for klass in register_classes:
		bpy.utils.register_class(klass)
		if hasattr(klass, "register"):
			klass.register()

def unregister():
	for klass in register_classes:
		bpy.utils.unregister_class(klass)
		if hasattr(klass, "unregister"):
			klass.unregister()

if __name__ == "__main__":
	register()
