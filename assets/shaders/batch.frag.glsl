#version 300 es
precision mediump float;

#define FLAG_SPRITE 1
#define FLAG_TEXT 2
#define FLAG_MERGE 4
#define FLAG_PICKER_BG 8

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
uniform vec2 u_mouse_pos;

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

// TODO: move picker background rending into a separate file

// Thanks to https://github.com/hughsk/glsl-dither
float dither4x4(vec2 position, float brightness) {
	int x = int(mod(position.x, 4.0));
	int y = int(mod(position.y, 4.0));
	int index = x + y * 4;
	float limit = 0.0;

	if (x < 8) {
		if (index == 0) limit = 0.0625;
		if (index == 1) limit = 0.5625;
		if (index == 2) limit = 0.1875;
		if (index == 3) limit = 0.6875;
		if (index == 4) limit = 0.8125;
		if (index == 5) limit = 0.3125;
		if (index == 6) limit = 0.9375;
		if (index == 7) limit = 0.4375;
		if (index == 8) limit = 0.25;
		if (index == 9) limit = 0.75;
		if (index == 10) limit = 0.125;
		if (index == 11) limit = 0.625;
		if (index == 12) limit = 1.0;
		if (index == 13) limit = 0.5;
		if (index == 14) limit = 0.875;
		if (index == 15) limit = 0.375;
	}

	return brightness < limit ? 0.0 : 1.0;
}

// Thanks to https://iquilezles.org for these funky SDF functions
float smin(float a, float b, float k) {
	k *= 6.0;
	float h = max( k-abs(a-b), 0.0 )/k;
	return min(a,b) - h*h*h*k*(1.0/6.0);
}
float circle(vec2 p, float r) {
	return length(p) - r;
}

float borders() {
	#define SMOOTH 0.05

	float left = smoothstep(0.22, 0.34, uv.x);
	float right = smoothstep(1.01, 0.9, uv.x);
	float bottom = smoothstep(-0.01, 0.1, uv.y);
	float top = smoothstep(1.01, 0.9, uv.y);
	return smin(smin(left, right, SMOOTH), smin(top, bottom, SMOOTH), SMOOTH);
}

// Thanks to https://math.stackexchange.com/a/2323106
float never_1(float x) {
	#define E 2.71828

	return 1.0 - pow(E, -x);
}

vec4 render_picker_bg() {
	vec2 mouse = vec2(u_mouse_pos.x, 1.0 - u_mouse_pos.y);

	float x = never_1(mouse.x) * 0.4;
	float f = mix(1.0, circle(uv - vec2(x, mouse.y), 0.06) * 3.0, u_factor);
	f = clamp(f, -1.0, 1.0);
	float b = borders();

	float result = 0.0;
	result = smin(f, b, 0.3);
	result = smoothstep(0.0, 0.5, result);

	float a = dither4x4(gl_FragCoord.xy, result);
	vec4 color = texture(u_texture1, vec2(uv.x, 1.0 - uv.y));
	return vec4(color.rgb * a, 1.0);
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

	if (HAS_FLAG(FLAG_PICKER_BG)) {
		return render_picker_bg();
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
