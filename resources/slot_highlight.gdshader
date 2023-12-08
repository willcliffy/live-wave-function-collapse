shader_type spatial;

uniform vec4 initial_color;
uniform float duration;
uniform float start_time; // Starting time of the fade-out animation


void fragment() {
    // Calculate the current time within the shader
    float current_time = TIME - start_time;
    
    // Calculate the elapsed time
    float elapsed_time = clamp(current_time, 0.0, duration);
    
    // Calculate the alpha value based on the elapsed time and duration
    float alpha = clamp(1.0 - elapsed_time / duration, 0.0, 1.0);
    
    // Blend the initial color with the calculated alpha
    vec4 final_color = initial_color * vec4(1.0, 1.0, 1.0, alpha);
    
    // Output the final color with modified alpha
    ALBEDO = final_color.rgb;
    ALPHA = final_color.a;
}
