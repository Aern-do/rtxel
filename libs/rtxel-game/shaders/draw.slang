import world;

struct Camera {
    float3 origin;
    float yaw;
    float pitch;

    float3 forward;
    float3 up;
    float aspect;
    float fov;

    uint frame_count;
};

[vk::binding(0, 0)]
StructuredBuffer<uint> grid;
[vk::binding(1, 0)]
StructuredBuffer<BrickMap> map;
[vk::binding(2, 0)]
WTexture2D<float4> outputTexture;

[vk::binding(0, 1)]
ConstantBuffer<Camera> camera;

float3 getRayDir(float2 uv) {
    float3 right = normalize(cross(camera.forward, camera.up));
    float3 up = normalize(cross(right, camera.forward));
    float halfTan = tan(camera.fov * 0.5);
    float2 ndc = (uv - 0.5) * 2.0;
    ndc.y = -ndc.y;

    return normalize(
        camera.forward
            + ndc.x * camera.aspect * halfTan * right
            + ndc.y * halfTan * up
    );
}

[shader("compute")]
[numthreads(8, 8)]
void main(uint3 threadId: SV_DispatchThreadID) {
    uint width, height;
    outputTexture.GetDimensions(width, height);

    if (threadId.x >= width || threadId.y >= height)
        return;

    float2 uv = float2(threadId.xy) / float2(width, height);
    float3 dir = getRayDir(uv);

    float3 color = float3(1.0);
    outputTexture.Store(threadId.xy, float4(color, 1.0));
}