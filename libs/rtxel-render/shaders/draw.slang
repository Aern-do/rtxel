Texture2D<float4> tex : register(t0, space0);
SamplerState tex_sampler : register(s1, space0);

struct VsOut {
    float4 pos : SV_Position;
    float2 uv  : TEXCOORD0;
};

[shader("vertex")]
VsOut vs(uint i : SV_VertexID) {
    float2 uv = float2(float((i << 1u) & 2u), float(i & 2u));
    float4 pos = float4(uv * 2.0 - 1.0, 0.0, 1.0);
    VsOut out;
    out.pos = pos;
    out.uv  = float2(uv.x, 1.0 - uv.y);
    return out;
}

[shader("fragment")]
float4 fs(VsOut in) : SV_Target {
    return tex.Sample(tex_sampler, in.uv);
}