#version 410

precision mediump float;

out vec4 fragColor;

uniform bool u_is_circle;

in vec2 v_tex_coord;

void main() {
    if (u_is_circle) {
        float R = 1.0;
        float R2 = 0.95;
        float dist = length(v_tex_coord);
        if (dist >= R || dist <= R2) {
            discard;
        }
        float sm = smoothstep(R,R-0.01,dist);
        float sm2 = smoothstep(R2,R2+0.01,dist);
        float alpha = sm*sm2;
        fragColor = vec4(0.0, 1.0, 0.0, alpha);
    } else {
        if (
            v_tex_coord.x > 0.01 && v_tex_coord.x < 0.99 
            && v_tex_coord.y > 0.01 && v_tex_coord.y < 0.99
        ) {
            discard;
        }
        fragColor = vec4(0.0, 1.0, 0.0, 1.0);
    }
}