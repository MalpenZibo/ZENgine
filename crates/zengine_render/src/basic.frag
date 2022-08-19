#version 410

precision mediump float;

out vec4 fragColor;

uniform vec4 u_tint;

uniform sampler2D u_diffuse;

in vec2 v_tex_coord;

void main() {
    fragColor = u_tint * texture(u_diffuse, v_tex_coord);
}