#version 450

layout(binding = 0) uniform sampler2D tex;

in vec4 color;
in vec2 tcoords;
in vec2 quad_coords;
in float select;

out vec4 frag;

void main() {
    float edge_proximity = select * 2.0 * max(abs(quad_coords.x - 0.5), abs(quad_coords.y - 0.5));
    if (edge_proximity > 0.95) {
        frag = vec4(color.rgb, 1.0);
    }
    else {
        frag = color * texture(tex, tcoords);
    }
}

