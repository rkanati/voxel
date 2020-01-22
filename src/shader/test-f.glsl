#version 450

//layout(binding = 0) uniform sampler2D tex;

in vec2 tcoords;
in float shade;

out vec4 frag;

void main() {
    frag = vec4(shade,shade,shade,1);//texture(tex, tcoords);
}

