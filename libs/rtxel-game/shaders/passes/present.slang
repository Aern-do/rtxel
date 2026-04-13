struct VertexOutput {
    float4 position : SV_Position;
    float2 uv : TEXCOORD0;
};

[vk::binding(0, 0)]
Texture2D<float4> inputTexture;
[vk::binding(1, 0)]
SamplerState inputSampler;

[shader("vertex")]
VertexOutput vs_main(uint vertexId : SV_VertexID) {
    // I've stole this from somewhere, but I have no fucking clue from where
    float x = float(int(vertexId & 1u)) * 4.0 - 1.0;
    float y = float(int(vertexId >> 1u)) * 4.0 - 1.0;

    VertexOutput output;
    output.position = float4(x, y, 0.0, 1.0);
    output.uv = float2((x + 1.0) * 0.5, (1.0 - y) * 0.5);
    return output;
}

[shader("fragment")]
float4 fs_main(VertexOutput input) : SV_Target {
    return inputTexture.Sample(inputSampler, input.uv);
}