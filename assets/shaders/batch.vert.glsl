#version 300 es
precision mediump float;

layout (location = 0) in vec2 a_pos;
layout (location = 1) in vec2 a_uv;
layout (location = 2) in float a_opacity;

uniform vec2 u_view_size_px;

out vec2 uv;
flat out float opacity;

void main() {
	uv = a_uv;
	opacity = a_opacity;

	gl_Position = vec4((a_pos / u_view_size_px * 2.0) + vec2(-1.0, -1.0), 0.0, 1.0);
}
// void main() {
// 	uv = a_uv;
//
// 	lowp vec2 vert_pos = a_pos + vec2(0.0, -1.0);
// 	lowp vec2 vert_scale = u_size_px / u_view_size_px * 2.0;
// 	lowp vec2 vert_offset = (u_pos_px / u_view_size_px * 2.0) * vec2(1.0, -1.0) + vec2(-1.0, 1.0);
//
// 	gl_Position = vec4(vert_pos * vert_scale + vert_offset, 0.0, 1.0);
// }
