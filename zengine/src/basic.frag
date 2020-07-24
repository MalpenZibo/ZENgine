#version 410

precision mediump float;

out vec4 fragColor;

uniform vec4 u_tint;

void main() {
    fragColor = u_tint;
}