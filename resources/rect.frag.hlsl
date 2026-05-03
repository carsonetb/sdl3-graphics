struct Input {
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


// Functions from https://iquilezles.org/articles/distfunctions2d/
float sdRoundedBox(float2 p, float2 b, float4 r) {
    r.xy = (p.x>0.0)?r.xy : r.zw;
    r.x  = (p.y>0.0)?r.x  : r.y;
    float2 q = abs(p)-b+r.x;
    return min(max(q.x,q.y),0.0) + length(max(q,0.0)) - r.x;
}

float sdCircle(float2 p, float r) {
    return length(p) - r;
}

float4 main(Input input) : SV_Target0 {
    float boxsd = sdRoundedBox(input.rect_position - input.rect_dimension / 2.0, input.rect_dimension / 2.0, input.radii);

    if (boxsd > 0.0) {
        float shadowsd = sdRoundedBox(input.rect_position - input.rect_dimension / 2.0 - input.shadow_offset, input.rect_dimension / 2.0, input.radii)
        float shadow_alpha = 1.0 - smoothstep(0.0, input.shadow_softness, shadowsd);

        return input.shadow_color * float4(1.0, 1.0, 1.0, shadow_alpha);
    }

    boxsd = abs(boxsd);
    if (boxsd < input.border_width) {
        return input.border_color;
    }

    return input.color;
}
