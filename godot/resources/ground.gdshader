shader_type spatial;

uniform float threshold : hint_range(0.0, 1.0, 0.1) = 0.6;
uniform float smoothing : hint_range(0.0, 0.1, 0.001) = 0.05;
uniform vec4 flat_color : source_color = vec4(0.0, 1.0, 0.0, 1.0);
uniform vec4 steep_color : source_color = vec4(0.5, 0.5, 0.5, 1.0);

varying vec3 interpolated_normal;

void vertex() {
	interpolated_normal = normalize(NORMAL);
}

void fragment() {    
    float dotProduct = dot(interpolated_normal, vec3(0.0, 1.0, 0.0));
    float smoothness = smoothstep(threshold - smoothing, threshold + smoothing, dotProduct);
    vec4 finalColor = mix(steep_color, flat_color, smoothness);

    ALBEDO = finalColor.rgb;
}