Texture2D atlas_texture : register(t0, space2);
SamplerState atlas_sampler : register(s0, space2);

struct Input {
    float4 position : SV_Position;
    float2 uv : TEXCOORD0;
    float4 color : COLOR0;
};

float4 main(Input input) : SV_Target {
    float distance = atlas_texture.Sample(atlas_sampler, input.uv).r;
    float smoothing = max(fwidth(distance), 0.0001)
    float alpha = smoothstep(0.5 - smoothing, 0.5 + smoothing, distance);

    // return float4(1.0, 1.0, 1.0, 1.0);
    return float4(input.color.rgb, input.color.a * alpha);
    // return float4(distance, distance, distance, 1.0);
}
