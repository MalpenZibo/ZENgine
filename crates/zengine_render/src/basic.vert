#version 410

in vec3 a_position;
in vec2 a_tex_coord;

uniform mat4 u_projection;
uniform mat4 u_model;

out vec2 v_tex_coord;

void main() {
    gl_Position = u_projection * u_model * vec4(a_position, 1.0);
    v_tex_coord = a_tex_coord;
}