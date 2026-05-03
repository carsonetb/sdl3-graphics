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

    float4 radii : TEXCOORD3;

    float border_width : TEXCOORD4;
    float4 border_color : TEXCOORD5;

    float4 shadow_color : TEXCOORD6;
    float2 shadow_offset : TEXCOORD7;
    float shadow_softness : TEXCOORD8;
};

struct Output {
    float4 color : TEXCOORD0;
    float4 position : SV_Position;
    float2 rect_position : TEXCOORD1;
    float2 rect_dimension : TEXCOORD2;

    float4 radii : TEXCOORD3;

    float border_width : TEXCOORD4;
    float4 border_color : TEXCOORD5;

    float4 shadow_color : TEXCOORD6;
    float2 shadow_offset : TEXCOORD7;
    float shadow_softness : TEXCOORD8;
};

Output main(Input input) {
    Output output;

    // Shadow stuff
    float2 padding_tl = max(float2(0.0, 0.0), -input.shadow_offset) + input.shadow_softness;
    float2 padding_br = max(float2(0.0, 0.0), input.shadow_offset) + input.shadow_softness;
    float2 is_br = step(0.5, input.base_pos);
    float2 is_tl = 1.0 - is_br;

    // scale base rectangle to actual dimensions.
    float2 scaled_pos = input.base_pos * input.rect_dimension;
    float2 expanded_pos = scaled_pos - (padding_tl * is_tl) + (padding_br * is_br);
    float2 pos = expanded_pos + input.rect_pos;

    output.position = mul(proj, float4(pos, 0.0, 1.0));
    output.color = input.rect_color;
    output.rect_position = expanded_pos;
    output.rect_dimension = input.rect_dimension;
    output.radii = input.radii;
    output.border_width = input.border_width;
    output.border_color = input.border_color;
    output.shadow_color = input.shadow_color;
    output.shadow_offset = input.shadow_offset;
    output.shadow_softness = input.shadow_softness;

    return output;
}
