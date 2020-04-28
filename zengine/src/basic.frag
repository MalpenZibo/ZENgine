precision mediump float;

uniform vec4 u_tint;

uniform sampler2D u_diffuse;

varying vec2 v_tex_coord;

void main() {
    gl_FragColor = u_tint * texture2D(u_diffuse, v_tex_coord);
}