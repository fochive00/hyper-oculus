#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(binding = 0) uniform UniformBufferObject {
    mat4 cam4_trans;
    vec4 cam4_col;
    vec4 cam4_row;
    mat4 cam3_trans;
    float cam4_const
} ubo;

layout(location = 0) in vec4 inPosition;
layout(location = 1) in vec3 inColor;

layout(location = 0) out vec3 fragColor;

void main() {
    gl_Position = ubo.cam3_trans * ((ubo.cam4_trans * inPosition + ubo.cam4_col) / (ubo.cam4_row * inPosition + cam4_const));

    fragColor = inColor;
}