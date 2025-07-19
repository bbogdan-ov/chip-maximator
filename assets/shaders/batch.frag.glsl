#version 300 es
precision mediump float;

#define FLAG_SPRITE 1
#define FLAG_TEXT 2
#define FLAG_MERGE 4

#define HAS_FLAG(FLAG) ((u_flags & FLAG) != 0)

#define BLEND_NORMAL 0
#define BLEND_SCREEN 1
#define BLEND_ADD 2
#define BLEND_OVERLAY 3

in vec2 uv;
flat in float opacity;

uniform sampler2D u_texture1;
uniform sampler2D u_texture2;

uniform vec2 u_view_size_px;
uniform int u_flags;
// Foreground tint
uniform vec3 u_foreground;
// Background tint
uniform vec4 u_background;
uniform int u_blend_mode;
uniform float u_factor;

out vec4 out_color;

vec3 blend(vec4 fg, vec4 bg) {
	// Normal
	if (u_blend_mode == BLEND_NORMAL)
		return mix(fg.rgb, bg.rgb, fg.a);

	// Screen
	if (u_blend_mode == BLEND_SCREEN)
		return 1.0 - (1.0 - fg.rgb) * (1.0 - bg.rgb);

	// Addition
	if (u_blend_mode == BLEND_ADD)
		return fg.rgb + bg.rgb;

	// Overlay
	if (u_blend_mode == BLEND_OVERLAY)
		return mix(
			2.0 * bg.rgb * fg.rgb,
			1.0 - 2.0 * (1.0 - bg.rgb) * (1.0 - fg.rgb),
			step(0.5, fg.rgb)
		);

	return vec3(1.0, 0.0, 1.0);
}

vec4 frag() {
	vec2 flipped_uv = vec2(uv.x, 1.0 - uv.y);

	// Sprite
	if (HAS_FLAG(FLAG_SPRITE)) {
		vec4 uv_color = texture(u_texture2, flipped_uv);

		if (uv_color.b > 0.0 || uv_color.a > 0.0) {
			// Use custom uv
			vec4 color = texture(u_texture1, vec2(uv_color.r, 1.0 - uv_color.g));
			color.a *= 1.0 - uv_color.b;
			return color;
		} else {
			// Use default uv
			return texture(u_texture1, flipped_uv);
		}
	}

	// Text
	if (HAS_FLAG(FLAG_TEXT)) {
		return texture(u_texture1, flipped_uv);
	}

	// Merge
	if (HAS_FLAG(FLAG_MERGE)) {
		vec4 bg_color = texture(u_texture1, uv);
		vec4 fg_color = texture(u_texture2, uv);
		vec3 rgb = blend(bg_color, fg_color);
		return mix(bg_color, vec4(rgb, max(bg_color.a, fg_color.a)), fg_color.a * u_factor);
	}

	// Purple color if something went wrong
	out_color = vec4(1.0, 0.0, 1.0, 1.0);
}

vec4 tint(vec4 color) {
	color.rgb *= u_foreground;
	return mix(color, u_background, (1.0 - color.a) * u_background.a);
}

void main() {
	vec4 color = tint(frag());
	color.a *= opacity;

	out_color = color;
}
