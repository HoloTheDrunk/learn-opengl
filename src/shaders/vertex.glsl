#version 450

layout(location = 0) in vec3 vPos;
layout(location = 1) in vec3 vNormal;

out vec3 normal;

float dist(vec3 pos) {
    return sqrt(pos.x * pos.x + pos.y * pos.y + pos.z * pos.z);
}

void main() {
    gl_Position = vec4(vPos / dist(vPos), 1.0);
    normal = normalize(vNormal);
}
