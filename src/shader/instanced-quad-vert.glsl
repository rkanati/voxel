#version 450

layout(location = 0) uniform mat4 model_to_clip;
layout(location = 1) uniform ivec3 selected;

layout(location = 2) uniform vec2 tex_tile_dims;
layout(location = 3) uniform vec2 tex_padding;
layout(location = 4) uniform vec2 tex_stride;

layout(location = 0) in ivec4 attr_pos_dir;
layout(location = 1) in  vec4 attr_color;
layout(location = 2) in ivec2 attr_tcoords;
layout(location = 3) in ivec2 attr_rotate_;

out vec4 color;
out vec2 tcoords;
out vec2 quad_coords;
out float select;

void main() {
    // currently in fan (cyclic) order
    const ivec3 POS_OFFSETS[] = ivec3[][] (
        ivec3[] (ivec3(1,1,1), ivec3(0,1,1), ivec3(0,0,1), ivec3(1,0,1)),
        ivec3[] (ivec3(0,1,1), ivec3(1,1,1), ivec3(1,1,0), ivec3(0,1,0)),
        ivec3[] (ivec3(1,1,1), ivec3(1,0,1), ivec3(1,0,0), ivec3(1,1,0)),
        ivec3[] (ivec3(1,1,0), ivec3(1,0,0), ivec3(0,0,0), ivec3(0,1,0)),
        ivec3[] (ivec3(0,0,1), ivec3(0,0,0), ivec3(1,0,0), ivec3(1,0,1)),
        ivec3[] (ivec3(0,1,1), ivec3(0,1,0), ivec3(0,0,0), ivec3(0,0,1))
    );

    // currently in fan (cyclic) order
    const ivec2 TEX_OFFSETS[] = ivec2[][] (
        ivec2[] (ivec2(1,1), ivec2(0,1), ivec2(0,0), ivec2(1,0)),
        ivec2[] (ivec2(0,1), ivec2(1,1), ivec2(1,0), ivec2(0,0)),
        ivec2[] (ivec2(1,1), ivec2(0,1), ivec2(0,0), ivec2(1,0)),
        ivec2[] (ivec2(1,1), ivec2(1,0), ivec2(0,0), ivec2(0,1)),
        ivec2[] (ivec2(0,1), ivec2(0,0), ivec2(1,0), ivec2(1,1)),
        ivec2[] (ivec2(1,1), ivec2(1,0), ivec2(0,0), ivec2(0,1))
    );

    ivec3 pos = attr_pos_dir.xyz;
    int   dir = attr_pos_dir.w;

    ivec3 coords = pos + POS_OFFSETS[dir][gl_VertexID];
    gl_Position = model_to_clip * vec4(vec3(coords), 1.0);

    float shade = 1.0 - (dir / 6.0);
    color = vec4(shade * attr_color.rgb, attr_color.a);

    int tc_rotate = attr_rotate_.x;
    int tc_index = (gl_VertexID + tc_rotate) & 3;
    vec2 quad_offs = vec2(TEX_OFFSETS[dir][tc_index]);
    vec2 offs = tex_tile_dims * vec2(quad_offs.x, 1.0 - quad_offs.y);
    tcoords = tex_padding
            + attr_tcoords * tex_stride
            + offs;
    quad_coords = quad_offs;

    if (pos == selected) {
        select = 1.0;
    }
    else {
        select = 0.0;
    }
}

