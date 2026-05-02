cbuffer UniformBlock : register(b0, space1) {
    float4x4 proj;
}

struct Input {
    // per-vertex
    float2 base_pos : POSITION0;

    // per-instance
    float2 rect_pos : TEXCOORD1;
    float2 rect_dimension : TEXCOORD2;
    float4 rect_color : COLOR0;
};

struct Output {
    float4 color : TEXCOORD0;
    float4 position : SV_Position;
};

Output main(Input input) {
    Output output;

    float2 scaled_pos = input.base_pos * input.rect_dimension;
    float2 pos = scaled_pos + input.rect_pos;

    output.position = mul(proj, float4(pos, 0.0, 1.0));
    output.color = input.rect_color;

    return output;
}
