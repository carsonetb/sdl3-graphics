cbuffer UniformBlock : register(b0, space1) {
    float4x4 proj;
}

struct Input {
    // Vertex data
    float2 position : POSITION0;

    // Instance data
    float4 transform : TEXCOORD0;
    float4 atlas_rect : TEXCOORD1;
    float4 color : COLOR0;
};

struct Output {
    float4 position : SV_Position;
    float2 uv : TEXCOORD0;
    float4 color : COLOR0;
};

Output main(Input input) {
    Output output;

    float2 pos = (input.position * input.transform.zw) + input.transform.xy;
    output.position = mul(proj, float4(pos, 0.0, 1.0));

    output.uv = lerp(input.atlas_rect.xy, input.atlas_rect.zw, input.position);
    output.color = input.color;

    return output;
}
