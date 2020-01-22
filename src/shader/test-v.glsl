#version 450

uniform mat4 model_to_clip;

in vec4 attr_coords;
in vec2 attr_tcoords;

out vec2 tcoords;
out float shade;

void main() {
    gl_Position = model_to_clip * vec4(attr_coords.xyz, 1.0);
    tcoords = attr_tcoords;
    shade = (attr_coords.w + 1.0) / 6.0;
}

